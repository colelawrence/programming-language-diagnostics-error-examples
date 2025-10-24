import { Events, State, makeSchema } from "@livestore/livestore";
import * as S from "effect/Schema";

export const SessionID = S.String.pipe(S.brand("SessionID"));
export type SessionID = typeof SessionID.Type;

export const tables = {
  sessions: State.SQLite.table({
    name: "sessions",
    columns: {
      id: State.SQLite.text({ schema: SessionID, primaryKey: true }),
      name: State.SQLite.text({ schema: S.String }),
      createdAt: State.SQLite.integer({ schema: S.Number }),
    },
  }),
};

export const events = {
  AddSession: Events.synced({
    name: "AddSession",
    schema: S.Struct({
      id: SessionID,
      name: S.String,
      createdAt: S.Number,
    }),
  }),
};

const materializers = {
  AddSession: ({
    id,
    name,
    createdAt,
  }: { id: SessionID; name: string; createdAt: number }) => {
    return [
      tables.sessions.insert({
        id,
        name,
        createdAt,
      }),
    ];
  },
};

const state = State.SQLite.makeState({ tables, materializers });

export const schema = makeSchema({
  events,
  state,
});
