import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';
import { createHash } from 'node:crypto';

const research = 'docs/research/developers-openai-com-56163ca2/blog-8caafe43';
const assets = JSON.parse(await readFile(`${research}/ASSET_MANIFEST.json`, 'utf8'));
const results = [];
for (let i = 0; i < assets.length; i += 4) {
  results.push(...await Promise.all(assets.slice(i, i + 4).map(async (asset) => {
    const response = await fetch(asset.url);
    if (!response.ok) throw new Error(`${response.status}: ${asset.url}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    await mkdir(dirname(asset.path), { recursive: true });
    await writeFile(asset.path, bytes);
    return { ...asset, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') };
  })));
}
await writeFile(`${research}/DOWNLOADED_ASSETS.json`, JSON.stringify(results, null, 2));
console.log(`Downloaded ${results.length} assets (${results.reduce((sum, item) => sum + item.bytes, 0)} bytes).`);
