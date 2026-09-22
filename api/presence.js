const { getPool, ensureSchema } = require('./_db');

module.exports = async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  await ensureSchema();

  const peerId = req.query.peer_id;
  if (!peerId) {
    return res.status(400).json({ error: 'Missing peer_id parameter' });
  }

  try {
    const pool = getPool();
    const { rows } = await pool.query(
      'SELECT peer_id, public_ip, public_port, local_ip, local_port, last_seen_at FROM presence_entries WHERE peer_id = $1 LIMIT 1',
      [peerId]
    );

    if (rows.length === 0) {
      return res.status(404).json({ online: false, peer_id: peerId });
    }

    const entry = rows[0];
    const isRecent = (Date.now() - Number(entry.last_seen_at)) < 60000; // 60s window
    return res.status(200).json({
      online: isRecent,
      ...entry,
    });
  } catch (err) {
    console.error('Presence lookup error:', err);
    return res.status(500).json({ error: 'Database query failed', details: err.message });
  }
};
