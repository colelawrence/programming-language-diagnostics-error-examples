import { sql } from "@livestore/livestore";
import { Schema } from "effect";
import { SessionID, tables } from "#src/livestore/schema.ts";

/**
 * Query all sessions using query builder
 */
export function allSessionsQuery() {
  return tables.sessions.select().orderBy("createdAt", "desc");
}

/**
 * Query a single session by ID using query builder
 */
export function sessionByIdQuery(sessionId: SessionID) {
  return tables.sessions
    .select()
    .where({ id: sessionId })
    .first({ fallback: () => undefined });
}

/**
 * Query sessions count using raw SQL
 */
export function sessionsCountQuery() {
  return {
    query: sql`SELECT COUNT(*) as count FROM sessions`,
    schema: Schema.Array(Schema.Struct({ count: Schema.Number })),
    bindValues: [],
  };
}

/**
 * Query recent sessions using raw SQL
 */
export function recentSessionsQuery(limit = 10) {
  return {
    query: sql`
      SELECT id, name, createdAt
      FROM sessions
      ORDER BY createdAt DESC
      LIMIT ?
    `,
    schema: Schema.Array(
      Schema.Struct({
        id: SessionID,
        name: Schema.String,
        createdAt: Schema.Number,
      }),
    ),
    bindValues: [limit],
  };
}
