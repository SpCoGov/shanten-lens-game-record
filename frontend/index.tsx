import React from "react";
import {createRoot, type Root} from "react-dom/client";
import GameRecordPage from "./pages/GameRecordPage";
import stylesheetText from "./pages/GameRecordPage.module.css?inline";
import {setHost, type PluginApi} from "./lib/host";
import {getRecord, setRecord, subscribeRecord, setOverrideArmed, type JsonValue} from "./lib/ipc";

function RecordsPage() {
    const record = React.useSyncExternalStore(subscribeRecord, getRecord, getRecord);
    return <GameRecordPage record={record} onChange={setRecord}/>;
}

export function activate(api: PluginApi) {
    if (typeof api.backend.onEvent !== "function" || typeof api.backend.request !== "function" || typeof api.ui.openPage !== "function") {
        throw new Error(api.i18n.t("game_record.host_upgrade_required"));
    }
    setHost(api);
    let disposed = false;
    let receivedRecord = false;
    let receivedOverride = false;
    const roots = new Set<Root>();
    const stylesheet = document.createElement("style");
    stylesheet.textContent = stylesheetText;
    document.head.appendChild(stylesheet);
    const unregister = api.ui.registerPage({id: "record", title: () => api.i18n.t("game_record.sidebar_title"), icon: "receipt_long", mount(container) {
        const root = createRoot(container);
        roots.add(root);
        root.render(<RecordsPage/>);
        return () => { if (roots.delete(root)) root.unmount(); };
    }});
    const updated = api.backend.onEvent("record.updated", data => {
        receivedRecord = true;
        setRecord(data as Record<string, JsonValue>);
        api.ui.openPage("record");
    });
    const override = api.backend.onEvent("record.override", value => {
        receivedOverride = true;
        setOverrideArmed(value === true);
    });
    void api.backend.invoke<{record: Record<string, JsonValue> | null; armed: boolean}>("record.get").then(snapshot => {
        if (disposed) return;
        if (!receivedRecord) {
            setRecord(snapshot.record);
            if (snapshot.record) api.ui.openPage("record");
        }
        if (!receivedOverride) setOverrideArmed(snapshot.armed);
    }).catch(error => {if (!disposed) api.ui.showToast(api.i18n.t("game_record.backend_failed", {reason: String(error)}), "error");});
    return () => {
        disposed = true;
        updated(); override(); unregister();
        roots.forEach(root => root.unmount());
        roots.clear();
        stylesheet.remove();
        setHost(undefined);
    };
}
