import {getHost} from "./host";

export type JsonValue = null | boolean | number | string | JsonValue[] | {[key: string]: JsonValue | undefined};
type CommandResult = {ok: boolean; reason?: string; error?: string};
let record: Record<string, JsonValue> | null = null;
let armed = false;
const listeners = new Set<() => void>();
const overrideListeners = new Set<(value: boolean) => void>();

export function getRecord() { return record; }
export function subscribeRecord(callback: () => void) { listeners.add(callback); return () => {listeners.delete(callback);}; }
export function setRecord(value: Record<string, JsonValue> | null) { record = value; listeners.forEach(callback => callback()); }
export function setOverrideArmed(value: boolean) { armed = value; overrideListeners.forEach(callback => callback(value)); }
export function subscribeBackendEvent(_name: "game_record_override_status", callback: (value: boolean) => void) {
    overrideListeners.add(callback);
    callback(armed);
    return () => {overrideListeners.delete(callback);};
}

export async function fetchGameRecord(gameUuid: string): Promise<CommandResult> {
    const uuid = gameUuid.trim();
    if (!uuid || uuid.length > 200) return {ok: false, reason: "invalid-payload"};
    try {
        const response = await getHost().backend.request<Record<string, unknown>>("records", ".lq.Lobby.fetchGameRecord", {
            clientVersionString: "StandaloneWindows_2022-0.16.273", gameUuid: uuid,
        });
        return {ok: !response.error, reason: response.error ? JSON.stringify(response.error) : undefined};
    } catch (error) { return {ok: false, reason: String(error)}; }
}

export async function overrideGameRecord(value: unknown): Promise<CommandResult> {
    return await getHost().backend.invoke<CommandResult>("record.override", value);
}
