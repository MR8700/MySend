const { getPool, ensureSchema } = require('./_db');

module.exports = async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  try {
    await ensureSchema();
    const pool = getPool();
    await pool.query('SELECT 1');
    return res.status(200).json({ status: 'ok', database: 'connected', schema: 'initialized', timestamp: Date.now() });
  } catch (err) {
    return res.status(200).json({ status: 'ok', database: 'disconnected', error: err.message, timestamp: Date.now() });
  }
};
