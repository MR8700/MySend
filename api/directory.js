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

  if (req.method === 'GET') {
    const query = req.query.query || req.query.q || '';
    const peerId = req.query.peer_id;

    try {
      if (peerId) {
        const { rows } = await pool.query(
          'SELECT peer_id, username, display_name, avatar_data_url, prekey_bundle_hex, registered_at, last_updated_at FROM directory_profiles WHERE peer_id = $1 LIMIT 1',
          [peerId]
        );
        if (rows.length === 0) {
          return res.status(200).json(null);
        }
        return res.status(200).json(rows[0]);
      }

      const pattern = `%${query.toLowerCase().trim()}%`;
      const { rows } = await pool.query(
        `SELECT peer_id, username, display_name, avatar_data_url, prekey_bundle_hex, registered_at, last_updated_at
         FROM directory_profiles
         WHERE LOWER(username) LIKE $1 OR LOWER(display_name) LIKE $1
         ORDER BY last_updated_at DESC
         LIMIT 50`,
        [pattern]
      );
      return res.status(200).json({ results: rows });
    } catch (err) {
      console.error('Directory search error:', err);
      return res.status(500).json({ error: 'Database query failed', details: err.message });
    }
  }

  if (req.method === 'POST') {
    try {
      const { peer_id, username, display_name, avatar_data_url, prekey_bundle_hex } = req.body;
      if (!peer_id || !username || !display_name || !prekey_bundle_hex) {
        return res.status(400).json({ error: 'Missing required fields' });
      }

      const now = Date.now();
      await pool.query(
        `INSERT INTO directory_profiles (peer_id, username, display_name, avatar_data_url, prekey_bundle_hex, registered_at, last_updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)
         ON CONFLICT (peer_id) DO UPDATE SET
           username = EXCLUDED.username,
           display_name = EXCLUDED.display_name,
           avatar_data_url = EXCLUDED.avatar_data_url,
           prekey_bundle_hex = EXCLUDED.prekey_bundle_hex,
           last_updated_at = EXCLUDED.last_updated_at`,
        [peer_id, username, display_name, avatar_data_url || null, prekey_bundle_hex, now]
      );

      return res.status(200).json({ success: true, peer_id });
    } catch (err) {
      console.error('Directory registration error:', err);
      return res.status(500).json({ error: 'Failed to register profile', details: err.message });
    }
  }

  return res.status(405).json({ error: 'Method not allowed' });
};
