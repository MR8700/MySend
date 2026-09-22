const { getPool, ensureSchema } = require('./_db');

module.exports = async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  await ensureSchema();
  const pool = getPool();
  const now = Date.now();

  // POST: Heartbeat / Online status update
  if (req.method === 'POST') {
    const { peer_id, status = 'online' } = req.body || {};
    if (!peer_id) {
      return res.status(400).json({ error: 'Missing peer_id' });
    }

    const forwarded = req.headers['x-forwarded-for'];
    const publicIp = forwarded ? forwarded.split(',')[0].trim() : (req.socket.remoteAddress || '127.0.0.1');
    const lastSeen = status === 'offline' ? 0 : now;

    try {
      await pool.query(
        `INSERT INTO presence_entries (peer_id, public_ip, public_port, local_ip, local_port, last_seen_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (peer_id) DO UPDATE 
         SET last_seen_at = $6, public_ip = $2`,
        [peer_id, publicIp, 0, null, null, lastSeen]
      );

      return res.status(200).json({ success: true, peer_id, online: status !== 'offline', timestamp: now });
    } catch (err) {
      console.error('Failed to update presence heartbeat:', err);
      return res.status(500).json({ error: 'Presence update failed', details: err.message });
    }
  }

  // GET: Presence status lookup
  if (req.method === 'GET') {
    if (req.query.ping) {
      return res.status(200).json({ status: 'ok', timestamp: now });
    }

    const peerIdsParam = req.query.peer_ids || req.query.peer_id;
    if (!peerIdsParam) {
      return res.status(400).json({ error: 'Missing peer_id or peer_ids parameter' });
    }

    const peerIds = peerIdsParam.split(',').map(s => s.trim()).filter(Boolean);
    if (peerIds.length === 0) {
      return res.status(200).json({ presence: {} });
    }

    try {
      const { rows } = await pool.query(
        'SELECT peer_id, public_ip, public_port, local_ip, local_port, last_seen_at FROM presence_entries WHERE peer_id = ANY($1)',
        [peerIds]
      );

      const presenceMap = {};
      for (const id of peerIds) {
        presenceMap[id] = false;
      }

      for (const row of rows) {
        // Online if heartbeat received within the last 30 seconds
        const isOnline = (now - Number(row.last_seen_at)) < 30000;
        presenceMap[row.peer_id] = isOnline;
      }

      if (peerIds.length === 1) {
        const singleId = peerIds[0];
        const foundRow = rows.find(r => r.peer_id === singleId);
        return res.status(200).json({
          online: presenceMap[singleId] || false,
          peer_id: singleId,
          last_seen_at: foundRow ? foundRow.last_seen_at : null,
        });
      }

      return res.status(200).json({
        presence: presenceMap,
        count: Object.keys(presenceMap).length,
      });
    } catch (err) {
      console.error('Presence lookup error:', err);
      return res.status(500).json({ error: 'Database query failed', details: err.message });
    }
  }

  return res.status(405).json({ error: 'Method not allowed' });
};
