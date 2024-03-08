const express = require('express');

const app = express();

const port = 8888;

app.get('/callback', (req, res) => {
  const code = req.query.code;

  if (!code) {
    return res.status(500).send('An error occured and you were not authorized');
  }

  return res
    .status(200)
    .send(`Successfully authorized, you can return to the terminal`);
});

app.listen(port, () => console.log(`Running on localhost:${port}`));
