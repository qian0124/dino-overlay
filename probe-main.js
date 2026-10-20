const { app } = require('electron');
const line = require('node:fs');
const msg = `[probe] pid=${process.pid} type=${JSON.stringify(process.type)} hasApp=${!!app} argv=${JSON.stringify(process.argv.slice(1))}\n`;
try { line.appendFileSync(process.env.DINO_PROBE_LOG, msg); } catch (e) {}
if (app && app.isReady && process.type !== 'renderer' && process.type !== 'gpu-process' && process.type !== 'utility' && process.type !== 'zygote') {
  app.whenReady().then(() => {
    try { line.appendFileSync(process.env.DINO_PROBE_LOG, `[probe] ready-to-continue pid=${process.pid}\n`); } catch(e){}
    setTimeout(() => process.exit(0), 1500);
  }).catch(()=>{});
} else {
  if (process.type !== undefined && process.type !== '') { try { line.appendFileSync(process.env.DINO_PROBE_LOG, `[probe] child exit pid=${process.pid}\n`); process.exit(0); } catch(e){} }
}
