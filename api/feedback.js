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
    const { sender_peer_id, rating, category, comment } = req.body;
    if (rating === undefined || rating === null) {
      return res.status(400).json({ error: 'Missing required field: rating' });
    }

    const pool = getPool();
    const id = randomUUID();
    const now = Date.now();

    await pool.query(
      `INSERT INTO user_feedbacks (id, sender_peer_id, rating, category, comment, created_at)
       VALUES ($1, $2, $3, $4, $5, $6)`,
      [id, sender_peer_id || null, Number(rating), category || 'general', comment || '', now]
    );

    return res.status(200).json({
      success: true,
      record: {
        id,
        sender_peer_id,
        rating,
        category: category || 'general',
        comment: comment || '',
        created_at: now,
      },
    });
  } catch (err) {
    console.error('Feedback submission error:', err);
    return res.status(500).json({ error: 'Failed to record feedback', details: err.message });
  }
};
