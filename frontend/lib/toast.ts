import {getHost} from "./host";

export function pushToast(message: string, kind: "info" | "success" | "error" = "info", _duration?: number) {
    getHost().ui.showToast(message, kind);
}
