import { API_URI } from "@/api/api";
import { sleep } from "@/helpers/sleep";
import { defineStore } from "pinia";
import { computed, ref, type ComputedRef, type Ref } from "vue";
import { useAuthStore } from "./auth";
import { randomString } from "@/helpers/random";
import { UserPermission } from "@/api/users";

export interface CurrentState {
  id: string;
  display: DisplayState;
}

export interface DisplayState {
  content: Record<string, string>;
  slide_type_id: string | null;
}

export const useStateStore = defineStore("state", () => {
  const WS_URI = "api/state";
  const RECONNECT_DELAY = 1000;
  const REQUIRED_SET_PERMISSIONS = UserPermission.OPERATION;
  const DEFAULT_PING_DELAY = 1000;
  const PING_ID_LENGTH = 8;

  let _ws: WebSocket | null = null;

  const _currentState: Ref<CurrentState> = ref({
    id: "",
    display: {
      content: {},
      slide_type_id: null,
    },
  });

  let _messageListener: ((evt: MessageEvent<any>) => void) | null = null;
  let _closeListener: ((evt: CloseEvent) => void) | null = null;
  let _errorListener: ((evt: Event) => void) | null = null;

  let _isConnecting: boolean = false;
  let _isConnected: boolean = false;
  let _isDisconnecting: boolean = false;
  let _isAuthenticated: boolean = false;

  let _debug: boolean = false;

  let _pingLoopDelay: number | null = DEFAULT_PING_DELAY;
  let _pingLoopTaskId: Symbol | undefined;
  let _pingLoopPromise: Promise<void> | undefined = undefined;

  function _waitForMessage<T>(
    extractFunction: (message: any) => T | undefined
  ): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const cleanupListeners = () => {
        _ws!.removeEventListener("error", errorListener);
        _ws!.removeEventListener("close", closeListener);
        _ws!.removeEventListener("message", messageListener);
      };

      const messageListener = (evt: MessageEvent<any>) => {
        try {
          const message_json = evt.data;
          const message = JSON.parse(message_json);
          const result = extractFunction(message);
          if (result !== undefined) {
            resolve(result);
            cleanupListeners();
          }
        } catch (e) {
          reject(e);
          cleanupListeners();
        }
      };

      const closeListener = (evt: CloseEvent) => {
        reject(evt);
        cleanupListeners();
      };

      const errorListener = (evt: Event) => {
        reject(evt);
        cleanupListeners();
      };

      _ws!.addEventListener("message", messageListener);
      _ws!.addEventListener("close", closeListener);
      _ws!.addEventListener("error", errorListener);
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
   * Connects (or reconnects) to the state websocket
   */
  async function connect(): Promise<void> {
    if (_isConnecting) {
      return;
    }
    _isConnecting = true;

    _isAuthenticated = false;

    // disconnect (if connected)
    await disconnect();
    _isDisconnecting = false;

    // connect
    while (_ws == null && !_isDisconnecting) {
      try {
        await _connectWs();
      } catch (e) {
        if (_debug) {
          console.error("Error occurred connecting to state websocket", e);
        }
        _ws = null;
        await sleep(RECONNECT_DELAY);
      }
    }

    if (_isDisconnecting) {
      return;
    }

    if (_ws == null) {
      throw new Error("Could not connect to state websocket");
    }

    _isConnected = true;

    const existingState = currentState.value;

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
        if (_debug) {
          console.error("Error parsing response from state websocket", e);
        }
        connect();
      }
    };
    _ws.addEventListener("message", _messageListener);

    // set up close listener
    _closeListener = async (evt: CloseEvent) => {
      // reconnect if we're not closing the connection on our end
      if (!_isDisconnecting) {
        connect();
      }
    };
    _ws.addEventListener("close", _closeListener);

    // set up error handler (reconnect on error)
    _errorListener = async (evt: Event) => {
      if (_debug) {
        console.error("Error occurred on state websocket", evt);
      }
      connect();
    };
    _ws.addEventListener("error", _errorListener);

    const authStore = useAuthStore();

    // get latest value
    await refresh();

    // automatically set session if user has required permissions
    if (((authStore.user?.permissions ?? 0) & REQUIRED_SET_PERMISSIONS) != 0) {
      await authenticate();

      // set state if state on server side is not set
      if (currentState.value.id == "" && existingState.id != "") {
        await setState(existingState);
      }
    }

    _startPingLoop();

    _isConnecting = false;
  }

  /**
   * Disconnects from the state websocket, cancelling any reconnect attempts
   */
  async function disconnect(): Promise<void> {
    _isDisconnecting = true;
    _isConnected = false;
    _isConnecting = false;
    _isAuthenticated = false;

    _endPingLoop();
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
   * The current state
   */
  const currentState: ComputedRef<CurrentState> = computed(
    () => _currentState.value
  );

  /**
   * Sends the user's session token to the state websocket
   */
  async function authenticate(): Promise<boolean> {
    const authStore = useAuthStore();

    const request = JSON.stringify({
      auth_token: authStore.token ?? "",
    });

    const authPromise = _waitForMessage((message) => message.auth);

    _ws?.send(request);

    const result = (await authPromise) as boolean;
    _isAuthenticated = result;

    return result;
  }

  /**
   * Sets a new state
   * @param state State to set
   */
  async function setState(state: CurrentState): Promise<CurrentState> {
    const request = JSON.stringify({
      state,
    });

    const setPromise = _waitForMessage((message) => message.state);

    _ws?.send(request);

    return await setPromise;
  }

  /**
   * Requests a refresh of the state
   */
  async function refresh(): Promise<CurrentState> {
    const request = JSON.stringify({ get: true });

    const refreshPromise = _waitForMessage((message) => message.state);

    _ws?.send(request);

    return await refreshPromise;
  }

  /**
   * Sends a ping to the server and waits for a response
   */
  async function ping(): Promise<void> {
    const value = randomString(PING_ID_LENGTH);

    const pingRequest = JSON.stringify({ ping: value });

    const pongPromise = _waitForMessage((message) =>
      message.pong == value ? message.pong : undefined
    );

    _ws?.send(pingRequest);

    return await pongPromise;
  }

  function _startPingLoop() {
    _pingLoopTaskId = Symbol();
    _pingLoopPromise = _pingLoop(_pingLoopTaskId);
  }

  async function _pingLoop(taskId: Symbol) {
    while (_pingLoopTaskId == taskId && _pingLoopDelay != null) {
      await ping();
      await sleep(_pingLoopDelay);
    }
  }

  async function _endPingLoop() {
    _pingLoopTaskId = undefined;
    await _pingLoopPromise;
  }

  /**
   * Sets the ping delay
   */
  function setPingDelay(ms: number | null) {
    _pingLoopDelay = ms;
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
    setPingDelay,
  };
});
