import { API_URI } from "@/api/api";
import { sleep } from "@/helpers/sleep";
import { defineStore } from "pinia";
import { computed, ref, type ComputedRef, type Ref } from "vue";
import { useAuthStore } from "./auth";
import { randomString } from "@/helpers/random";

export interface DisplayState {
  id: string;
  content: Record<string, string>;
  slide_type_id: string | null;
}

export const useDisplayStateStore = defineStore("displayState", () => {
  const WS_URI = "api/display-state";
  const RECONNECT_DELAY = 1000;

  let _ws: WebSocket | null = null;

  const _currentState: Ref<DisplayState> = ref({
    id: "",
    content: {},
    slide_type_id: null,
  });

  let _messageListener: ((evt: MessageEvent<any>) => void) | null = null;
  let _closeListener: ((evt: CloseEvent) => void) | null = null;
  let _errorListener: ((evt: Event) => void) | null = null;

  let _isConnected: boolean = false;
  let _stopRetries: boolean = false;

  function _waitForEvent<TEventType extends Event, TValue>(
    event: "open" | "error" | "message" | "close",
    handler: (evt: TEventType) => TValue | undefined
  ): Promise<TValue> {
    return new Promise<TValue>((resolve, reject) => {
      const messageListener = (evt: TEventType) => {
        try {
          const result = handler(evt);
          if (result !== undefined) {
            resolve(result);
          }
        } catch (e) {
          reject(e);
        }
        _ws!.removeEventListener("message", messageListener as any);
      };
      _ws!.addEventListener("message", messageListener as any);
    });
  }

  async function _connectWs() {
    // connect to websocket
    _ws = new WebSocket(`${API_URI}${WS_URI}`);

    // wait for connection to open
    await new Promise<void>((resolve, reject) => {
      const removeListeners = () => {
        _ws!.removeEventListener("open", openListener);
        _ws!.removeEventListener("error", errorListener);
      };

      const errorListener = (e: Event) => {
        reject(e);
        removeListeners();
      };
      _ws!.addEventListener("error", errorListener);

      const openListener = () => {
        resolve();
        removeListeners();
      };
      _ws!.addEventListener("open", openListener);

      if (_ws!.readyState === WebSocket.OPEN) {
        resolve();
        removeListeners();
      }
    });
  }

  /**
   * Connects (or reconnects) to the display state websocket
   */
  async function connect(): Promise<void> {
    // disconnect (if connected)
    disconnect();
    _stopRetries = false;

    // connect
    while (_ws == null && !_stopRetries) {
      try {
        await _connectWs();
      } catch (e) {
        console.error(
          "Error occurred connecting to display state websocket",
          e
        );
        _ws = null;
        await sleep(RECONNECT_DELAY);
      }
    }

    if (_stopRetries) {
      return;
    }

    if (_ws == null) {
      throw new Error("Could not connect to display state websocket");
    }

    _isConnected = true;

    // set up message listener
    _messageListener = async (evt: MessageEvent<any>) => {
      try {
        const response = JSON.parse(evt.data);

        // set state if state changed
        if (response.state !== undefined) {
          _currentState.value = response.state;
        }

        // respond to pings
        if (response.ping !== undefined) {
          _ws?.send(JSON.stringify({ pong: response.ping }));
        }
      } catch (e) {
        console.error("Error parsing response from display state websocket", e);
      }
    };
    _ws.addEventListener("message", _messageListener);

    // set up close listener
    _closeListener = async (evt: CloseEvent) => {
      disconnect();
      _stopRetries = false;

      // reconnect if not closing
      await sleep(RECONNECT_DELAY);
      if (!_stopRetries) {
        await connect();
      }
    };
    _ws.addEventListener("close", _closeListener);

    // set up error handler (reconnect on error)
    _errorListener = async (evt: Event) => {
      console.error("Error occurred on display state websocket", evt);
      disconnect();
      _stopRetries = false;

      // reconnect if not closing
      await sleep(RECONNECT_DELAY);
      if (!_stopRetries) {
        await connect();
      }
    };
    _ws.addEventListener("error", _errorListener);

    // get latest value
    await refresh();
  }

  /**
   * Disconnects from the display state websocket
   */
  function disconnect(): void {
    _stopRetries = true;
    _isConnected = false;

    _ws?.removeEventListener("message", _messageListener!);
    _ws?.removeEventListener("close", _closeListener!);
    _ws?.removeEventListener("error", _errorListener!);
    _ws?.close();
    _ws = null;
  }

  /**
   * Whether the websocket is connected
   */
  const connected: ComputedRef<boolean> = computed(() => _isConnected);

  /**
   * The current display state
   */
  const currentState: ComputedRef<DisplayState> = computed(
    () => _currentState.value
  );

  /**
   * Sends the user's session token to the display state websocket
   */
  async function authenticate(): Promise<void> {
    const authStore = useAuthStore();

    const request = JSON.stringify({
      auth_token: authStore.token ?? "",
    });

    const authPromise = _waitForEvent("message", (evt: MessageEvent<any>) => {
      const response = JSON.parse(evt.data);
      return response.auth;
    });

    _ws?.send(request);

    return await authPromise;
  }

  /**
   * Sets a new state for the display
   * @param state State to set
   */
  async function setState(state: DisplayState): Promise<void> {
    const request = JSON.stringify({
      state,
    });

    const setPromise = _waitForEvent("message", (evt: MessageEvent<any>) => {
      const response = JSON.parse(evt.data);
      return response.state;
    });

    _ws?.send(request);

    await setPromise;
  }

  /**
   * Requests a refresh of the display state
   */
  async function refresh(): Promise<void> {
    const request = JSON.stringify({ get: true });

    const refreshPromise = _waitForEvent(
      "message",
      (evt: MessageEvent<any>) => {
        const response = JSON.parse(evt.data);
        return response.state;
      }
    );

    _ws?.send(request);

    await refreshPromise;
  }

  /**
   * Sends a ping to the server and waits for a response
   */
  async function ping(): Promise<void> {
    const value = randomString(16);

    const pingRequest = JSON.stringify({ ping: value });

    const pongPromise = _waitForEvent("message", (evt: MessageEvent<any>) => {
      const response = JSON.parse(evt.data);

      if (response.pong !== undefined && response.pong == value) {
        return response.pong;
      } else {
        return undefined;
      }
    });

    _ws?.send(pingRequest);

    return await pongPromise;
  }

  return {
    connect,
    disconnect,
    connected,
    currentState,
    authenticate,
    setState,
    refresh,
    ping,
  };
});
