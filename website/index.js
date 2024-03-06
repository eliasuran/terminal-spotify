const express = require('express');

const app = express();

const port = 8888;

app.get('/callback', (req, res) => {
  const code = req.query.code;
  res
    .status(200)
    .send(`your auth code is ${code}\nyou can return to the terminal`);
});

app.listen(port, () => console.log(`Running on localhost:${port}`));
