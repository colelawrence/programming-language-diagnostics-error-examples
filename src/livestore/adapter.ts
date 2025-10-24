import { makePersistedAdapter } from "@livestore/adapter-web";
// @ts-ignore
import LiveStoreSharedWorker from "@livestore/adapter-web/shared-worker?sharedworker";
// @ts-ignore
import LiveStoreWorker from "./livestore.worker.ts?worker";

export const liveStoreAdapter = makePersistedAdapter({
  storage: { type: "opfs" },
  worker: LiveStoreWorker,
  sharedWorker: LiveStoreSharedWorker,
});
