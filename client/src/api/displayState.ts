import { useAuthStore } from "@/stores/auth";
import { API_URI } from "./api";
import { computed, nextTick, ref, type ComputedRef, type Ref } from "vue";
import { sleep } from "@/helpers/sleep";

export interface DisplayState {
  id: string;
  content: Record<string, string>;
  slide_type_id: string | null;
}

export class DisplayStateConnection {
  readonly WS_URI = "api/display-state";
  readonly RECONNECT_DELAY = 1000;

  private _ws: WebSocket | null = null;

  private _currentState: Ref<DisplayState> = ref({
    id: "",
    content: {},
    slide_type_id: null,
  });

  private _messageListener: ((evt: MessageEvent<any>) => void) | null = null;
  private _closeListener: ((evt: CloseEvent) => void) | null = null;
  private _errorListener: ((evt: Event) => void) | null = null;

  private _isConnected: boolean = false;
  private _isClosing: boolean = false;

  static async connect(): Promise<DisplayStateConnection> {
    // debugger;
    const conn = new DisplayStateConnection();

    await conn.reconnect();

    return conn;
  }

  async connectWs() {
    // connect to websocket
    this._ws = new WebSocket(`${API_URI}${this.WS_URI}`);

    // wait for connection to open
    await new Promise<void>((resolve, reject) => {
      const removeListeners = () => {
        this._ws!.removeEventListener("open", openListener);
        this._ws!.removeEventListener("error", errorListener);
      };

      const openListener = () => {
        resolve();
        removeListeners();
      };
      this._ws!.addEventListener("open", openListener);

      const errorListener = (e: Event) => {
        reject(e);
        removeListeners();
      };
      this._ws!.addEventListener("error", errorListener);

      if (this._ws!.readyState === WebSocket.OPEN) {
        resolve();
        removeListeners();
      }
    });
  }

  async reconnect() {
    // disconnect (if connected)
    this.disconnect();
    this._isClosing = false;

    // connect
    while (this._ws == null && !this._isClosing) {
      try {
        await this.connectWs();
      } catch (e) {
        console.error(
          "Error occurred connecting to display state websocket",
          e
        );
        this._ws = null;
        await sleep(this.RECONNECT_DELAY);
      }
    }

    if (this._isClosing) {
      return;
    }

    if (this._ws == null) {
      throw new Error("Could not connect to display state websocket");
    }

    this._isConnected = true;

    // set up message listener
    this._messageListener = async (evt: MessageEvent<any>) => {
      try {
        const response = JSON.parse(evt.data);

        if (response.state != undefined) {
          this._currentState.value = response.state;
        }
      } catch (e) {
        console.error("Error parsing response from display state websocket", e);
      }
    };
    this._ws.addEventListener("message", this._messageListener);

    // set up close listener
    this._closeListener = async (evt: CloseEvent) => {
      this.disconnect();

      // reconnect if not closing
      await sleep(this.RECONNECT_DELAY);
      if (!this._isClosing) {
        await this.reconnect();
      }
    };
    this._ws.addEventListener("close", this._closeListener);

    // set up error handler (reconnect on error)
    this._errorListener = async (evt: Event) => {
      console.error("Error occurred on display state websocket", evt);
      this.disconnect();

      // reconnect if not closing
      await sleep(this.RECONNECT_DELAY);
      if (!this._isClosing) {
        await this.reconnect();
      }
    };
    this._ws.addEventListener("error", this._errorListener);

    // get latest value
    const initialLoadPromise = new Promise<void>((resolve, _) => {
      const initialLoadListener = (evt: MessageEvent<any>) => {
        resolve();
        this._ws!.removeEventListener("message", initialLoadListener);
      };
      this._ws!.addEventListener("message", initialLoadListener);
    });
    this.refresh();
    await initialLoadPromise;
  }

  disconnect(): void {
    this._isClosing = true;
    this._isConnected = false;

    this._ws?.removeEventListener("message", this._messageListener!);
    this._ws?.removeEventListener("close", this._closeListener!);
    this._ws?.removeEventListener("error", this._errorListener!);
    this._ws?.close();
    this._ws = null;
  }

  connected: ComputedRef<boolean> = computed(() => this._isConnected);

  currentState: ComputedRef<DisplayState> = computed(
    () => this._currentState.value
  );

  async authenticate(): Promise<void> {
    const authStore = useAuthStore();

    const request = {
      auth_token: authStore.token,
    };

    this._ws?.send(JSON.stringify(request));
  }

  setState(state: DisplayState): void {
    const authStore = useAuthStore();

    const request = {
      state,
    };

    this._ws?.send(JSON.stringify(request));
  }

  refresh(): void {
    this._ws?.send(JSON.stringify({ get: true }));
  }
}
