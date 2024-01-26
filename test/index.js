import express from 'express';

const app = express();

app.use(express.json());
app.use(express.text());

app.get('/', (req, res) => {
	res.send('hello, world!');
});

app.get('/status/:status', (req, res) => {
	res.status(parseInt(req.params.status)).send();
});

app.post('/json', (req, res) => {
	res.json(req.body);
});

app.post('/text', (req, res) => {
	res.send(req.body);
});

app.listen(1337);
