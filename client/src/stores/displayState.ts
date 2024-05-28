import { API_URI } from "@/api/api";
import { sleep } from "@/helpers/sleep";
import { defineStore } from "pinia";
import { computed, ref, type ComputedRef, type Ref } from "vue";
import { useAuthStore } from "./auth";

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

  async function _connectWs() {
    // connect to websocket
    _ws = new WebSocket(`${API_URI}${WS_URI}`);

    // wait for connection to open
    await new Promise<void>((resolve, reject) => {
      const removeListeners = () => {
        _ws!.removeEventListener("open", openListener);
        _ws!.removeEventListener("error", errorListener);
      };

      const openListener = () => {
        resolve();
        removeListeners();
      };
      _ws!.addEventListener("open", openListener);

      const errorListener = (e: Event) => {
        reject(e);
        removeListeners();
      };
      _ws!.addEventListener("error", errorListener);

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

        if (response.state != undefined) {
          _currentState.value = response.state;
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
    const initialLoadPromise = new Promise<void>((resolve, _) => {
      const initialLoadListener = (evt: MessageEvent<any>) => {
        resolve();
        _ws!.removeEventListener("message", initialLoadListener);
      };
      _ws!.addEventListener("message", initialLoadListener);
    });
    refresh();
    await initialLoadPromise;
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

    const request = {
      auth_token: authStore.token ?? "",
    };

    const authPromise = new Promise<void>((resolve) => {
      const authListener = (evt: MessageEvent<any>) => {
        try {
          const response = JSON.parse(evt.data);

          if (response.auth != undefined) {
            resolve(response.auth);
          }
        } catch (e) {
          console.error(
            "Error parsing response from display state websocket",
            e
          );
        }
        _ws!.removeEventListener("message", authListener);
      };
      _ws!.addEventListener("message", authListener);
    });

    _ws?.send(JSON.stringify(request));

    return await authPromise;
  }

  /**
   * Sets a new state for the display
   * @param state State to set
   */
  function setState(state: DisplayState): void {
    const authStore = useAuthStore();

    const request = {
      state,
    };

    _ws?.send(JSON.stringify(request));
  }

  /**
   * Requests a refresh of the display state
   */
  function refresh(): void {
    _ws?.send(JSON.stringify({ get: true }));
  }

  return {
    connect,
    disconnect,
    connected,
    currentState,
    authenticate,
    setState,
    refresh,
  };
});
