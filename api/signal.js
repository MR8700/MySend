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

  if (req.method === 'POST') {
    try {
      const { target_peer_id, sender_peer_id, signal } = req.body || {};
      if (!target_peer_id || !signal) {
        return res.status(400).json({ error: 'Missing target_peer_id or signal payload' });
      }

      const insertRes = await pool.query(
        `INSERT INTO call_signals (target_peer_id, sender_peer_id, signal_payload, created_at)
         VALUES ($1, $2, $3, $4) RETURNING id`,
        [target_peer_id, sender_peer_id || '', JSON.stringify(signal), now]
      );

      // Clean up old signals (> 3 minutes) asynchronously
      pool.query('DELETE FROM call_signals WHERE created_at < $1', [now - 180000]).catch(() => {});

      return res.status(200).json({ success: true, signal_id: insertRes.rows[0].id });
    } catch (err) {
      console.error('Failed to post call signal:', err);
      return res.status(500).json({ error: 'Signal delivery failed', details: err.message });
    }
  }

  if (req.method === 'GET') {
    const peerId = req.query.peer_id;
    if (!peerId) {
      return res.status(400).json({ error: 'Missing peer_id query parameter' });
    }

    try {
      // Fetch and atomically drain pending signals for this peer
      const { rows } = await pool.query(
        `SELECT id, sender_peer_id, signal_payload, created_at 
         FROM call_signals 
         WHERE target_peer_id = $1 AND created_at >= $2 
         ORDER BY id ASC`,
        [peerId, now - 120000]
      );

      if (rows.length > 0) {
        const ids = rows.map(r => r.id);
        await pool.query('DELETE FROM call_signals WHERE id = ANY($1)', [ids]);
      }

      const signals = rows.map(r => {
        let payload = r.signal_payload;
        if (typeof payload === 'string') {
          try { payload = JSON.parse(payload); } catch (_) {}
        }
        return {
          ...payload,
          senderPeerId: r.sender_peer_id || payload.senderPeerId,
          dbSignalId: r.id,
          createdAt: r.created_at
        };
      });

      return res.status(200).json({ signals });
    } catch (err) {
      console.error('Failed to drain call signals:', err);
      return res.status(500).json({ error: 'Signal drain failed', details: err.message });
    }
  }

  return res.status(405).json({ error: 'Method not allowed' });
};
