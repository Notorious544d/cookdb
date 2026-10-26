let dbPromise;

export function openDb() {
  dbPromise ??= (async () => {
    const [SQL, buf] = await Promise.all([
      initSqlJs({ locateFile: (f) => `js/${f}` }), // global from sql-wasm.js
      fetch('./recipes.db').then((r) => {
        if (!r.ok) throw new Error(`recipes.db: HTTP ${r.status}`);
        return r.arrayBuffer();
      }),
    ]);
    return new SQL.Database(new Uint8Array(buf));
  })();
  return dbPromise;
}

export function query(db, sql, params = []) {
  const stmt = db.prepare(sql);
  stmt.bind(params);
  const rows = [];
  while (stmt.step()) rows.push(stmt.getAsObject());
  stmt.free();
  return rows;
}