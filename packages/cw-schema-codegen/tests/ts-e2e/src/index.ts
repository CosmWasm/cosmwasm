import * as gen from './gen';
import process from 'node:process';

async function read(stream: NodeJS.ReadStream): Promise<string> {
    const chunks: any[] = [];
    for await (const chunk of stream) chunks.push(chunk);
    return Buffer.concat(chunks).toString('utf8');
}

const stdinString = await read(process.stdin);

// Match based on the argument, then attempt to deserialize and validate. Then re-serialize and emit.
const typeName = process.argv[2];
const deserialized = JSON.parse(stdinString);

let validated = gen[typeName].parse(deserialized);
console.error(stdinString);
console.error(deserialized);
console.error(validated);

const outputStream = process.stdout;
outputStream.write(JSON.stringify(validated));
