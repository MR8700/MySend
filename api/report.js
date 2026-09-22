const { getPool, ensureSchema } = require('./_db');
const { randomUUID } = require('crypto');

module.exports = async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  await ensureSchema();

  try {
    const { reporter_peer_id, target_peer_id, reason, category, comment } = req.body;
    if (!reporter_peer_id || !target_peer_id || !reason) {
      return res.status(400).json({ error: 'Missing required fields' });
    }

    const pool = getPool();
    const id = randomUUID();
    const now = Date.now();

    await pool.query(
      `INSERT INTO user_reports (id, reporter_peer_id, target_peer_id, reason, category, comment, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [id, reporter_peer_id, target_peer_id, reason, category || 'other', comment || '', now]
    );

    // Count reports on target
    const countRes = await pool.query(
      'SELECT COUNT(*) FROM user_reports WHERE target_peer_id = $1',
      [target_peer_id]
    );
    const count = parseInt(countRes.rows[0].count, 10);
    let autoQuarantined = false;

    if (count >= 5) {
      await pool.query(
        `INSERT INTO banned_users (peer_id, reason, banned_at, report_count, is_automatic)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (peer_id) DO UPDATE SET report_count = EXCLUDED.report_count`,
        [target_peer_id, `Auto-quarantined: ${count} abuse reports`, now, count]
      );
      autoQuarantined = true;
    }

    return res.status(200).json({
      success: true,
      auto_quarantined: autoQuarantined,
      message: autoQuarantined
        ? 'Report submitted; user auto-quarantined'
        : 'Report submitted successfully',
    });
  } catch (err) {
    console.error('Report submission error:', err);
    return res.status(500).json({ error: 'Failed to record report', details: err.message });
  }
};
