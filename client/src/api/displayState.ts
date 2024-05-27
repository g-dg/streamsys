import { API_URI } from "./api";
import type { SlideType } from "./slideType";

export interface DisplayState {
  id: string;
  content: Record<string, string>;
  slide_type: SlideType | null;
}

export class DisplayStateConnection {
  private ws: WebSocket;

  constructor() {
    this.ws = new WebSocket(`${API_URI}/display-state`);
  }

  setState(state: DisplayState) {}

  async waitForUpdate(): Promise<DisplayState> {
    return new Promise((resolve, reject) => {});
  }
}
