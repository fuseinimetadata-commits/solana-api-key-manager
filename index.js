const http = require('http');

const PORT = process.env.PORT || 3000;

const server = http.createServer((req, res) => {
  if (req.url === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', service: 'solana-api-key-manager' }));
    return;
  }
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    name: 'NEXUS Solana API Key Manager',
    description: 'On-chain API key management for Solana dApps',
    version: '1.0.0',
    status: 'running'
  }));
});

server.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
