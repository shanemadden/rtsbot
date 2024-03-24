const { spawnSync } = require('child_process');
const util = require('util');
const readline = require('readline');

const axios = require('axios');
const argv = require('yargs')
  .option('speed', {
    describe: 'target tick duration, in milliseconds',
    default: 1000,
  })
  .option('compose', {
    describe: 'docker-compose file to use',
    default: 'docker-compose.yml',
  })
  .option('max', {
    describe: 'number of ticks to stop after reaching',
    default: 50000,
  })
  .option('level', {
    describe: 'room controller level to stop after reaching',
    default: 5,
  })
  .option('map', {
    describe: 'map from assets directory to use',
  })
  .option('room', {
    describe: 'room to spawn in and monitor',
    default: 'W7N3',
  })
  .option('steamid', {
    describe: 'steam id (from https://store.steampowered.com/account/) to associate to bot account',
    type: 'string',
  })
  .demandOption('steamid')
  .option('leaverunning', {
    describe: 'leave server running after reaching goal',
    type: 'boolean',
    default: false,
  })
  .argv;

const sleep = util.promisify(setTimeout)

async function cli(expr, show) {
  // print input command
  if (show) console.log('>', expr);
  let { data } = await axios.post('http://localhost:21028/cli', expr);
  // trim trailing spaces
  if (typeof data === 'string') data = data.replace(/^\s+|\s+$/g, '');
  // print response
  if (show) console.log('<', data);
  return data
}

async function start_server() {
  console.log("starting docker compose...");
  await spawnSync('docker', ['compose', '-f', argv.compose, 'up', '-d', '--build'], { stdio: 'inherit' });
  while (true) {
    try {
      let { data } = await axios.get('http://localhost:21028/greeting');
      console.log("server alive:", data.split("\n")[0]);
      break
    } catch (e) {
      console.log('waiting for server (see docker logs)...');
      await sleep(5000);
    }
  }
}

async function reset_server() {
  await cli('system.setTickDuration(50000)', true);
  // wait a moment for that to take effect before reset, otherwise it frequently
  // pauses on tick 2
  await sleep(1000);
  await cli('system.resetAllData().then(() => system.pauseSimulation())', true);
  await sleep(1000);
  // print where we paused - hopefully 1!
  await cli('storage.env.get(storage.env.keys.GAMETIME)', true);
}

async function load_map(map_name) {
  // set a key to mark that the loading process has initiated but not yet completed
  // the import command times out at 2 minutes so we mark this key as 1 after it completes
  await cli('storage.env.set("MAP_LOADED", 0)', true);
  console.log("loading map...");
  await cli(`utils.importMap("http://maps-nginx/${map_name}.json").then(() => storage.env.set("MAP_LOADED", 1))`, true).catch((e) => {
    console.log("request failed (timeout expected for large maps)")
  })
  while (true) {
    let loaded = await cli('storage.env.get("MAP_LOADED")', true).catch((e) => {
      console.log("still not responsive, waiting..")
    });
    await sleep(5000);
    if (loaded) {
      console.log("map load completed!");
      await sleep(5000);
      break
    }
  }
}

async function prep_server() {
  console.log("preparing server...");
  await cli(`system.setTickDuration(${argv.tickspeed})`, true);
  // map just loaded, update terrain data instead of waiting for cron job
  await cli('map.updateTerrainData()', true);
  await cli('system.runCronjob("roomsForceUpdate")', true);
  await cli('system.runCronjob("fixAllowed")', true);
  // nuke all other bots before spawning ours in auto mode in the target room
  await cli('utils.removeBots()', true);
  await cli(`bots.spawn("rtsbot", "${argv.room}", { username: "rtsbot", auto: true })`, true);
  console.log("configuring bot account for login...");
  await cli('setPassword("rtsbot", "insecure")', false);
  await cli(`storage.db["users"].update({ username: "rtsbot" }, { $set: { cpuAvailable: 10000, steam: { id: "${argv.steamid}" } } })`, false);
  await cli('storage.db["users"].update({ username: "rtsbot" }, { $set: { badge: { type: 15, color1: "#000000", color2: "#0066ff", color3: "#3e3e3e", param: 0, flip: false } } })', false);
  await sleep(1000);
}

async function run_deploy() {
  console.log("invoking deploy script for localhost...");
  await spawnSync('npm', ['run', 'deploy', '--', '--server', 'localhost'], { stdio: 'inherit', shell: true, encoding:'utf-8' });
  console.log("waiting 10 seconds for code deploy to settle...");
  await sleep(10000);
}

function wait_for_input() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise(resolve => rl.question(`ready! place a spawn construction site then press enter.. > `, ans => {
    rl.close();
    resolve(ans);
  }))
}

async function run_benchmark() {
  console.time('benchmark');
  await cli('system.resumeSimulation()', true);
  let spawn_logged = false;
  while (true) {
    if (!spawn_logged) {
      let spawn_up = await cli(`storage.db["rooms.objects"].findOne({ room: "${argv.room}", type: "spawn" })`, false);
      if (spawn_up) {
        let x = await cli(`storage.db["rooms.objects"].findOne({ room: "${argv.room}", type: "spawn" }).get("x")`, false);
        let y = await cli(`storage.db["rooms.objects"].findOne({ room: "${argv.room}", type: "spawn" }).get("y")`, false);
        console.log(`spawn has been placed at ${x}, ${y}`);
        spawn_logged = true;
      }
    }
    let tick_number = await cli('storage.env.get(storage.env.keys.GAMETIME)', false);
    let controller_level = await cli(`storage.db["rooms.objects"].findOne({room: "${argv.room}", type: "controller"}).get("level")`, false);
    let gcl = await cli(`storage.db["users"].findOne({ username: "rtsbot" }).get("gcl")`, false);
    console.log(`tick ${tick_number} / ${argv.maxtick}, level ${controller_level} / ${argv.level}, gcl ${gcl}`);
    if (tick_number > argv.maxtick || controller_level >= argv.level) break;
    await sleep(5000);
  }
  console.timeEnd('benchmark');
}

async function post_run_cleanup() {
  console.log("tailing history.log:");
  await spawnSync('docker', ['compose', '-f', argv.compose, 'exec', 'screeps', 'tail', '/screeps/logs/history.log'], { stdio: 'inherit' });
  if (argv.leaverunning) {
    console.log("not stopping containers due to --leaverunning - don't forget to stop 'em!");
  } else {
    await spawnSync('docker', ['compose', '-f', argv.compose, 'down', '-v'], { stdio: 'inherit' });
  }
}

async function run() {
  console.time('environment-prep');
  // start docker-compose and wait until it's responsive
  await start_server();
  // run the reset and pause the server
  await reset_server();
  // load the map if one's been passed
  if (argv.map) await load_map(argv.map);
  // create user account and prep for run
  await prep_server();
  // build and deploy code using the credentials just set up
  await run_deploy();
  console.timeEnd('environment-prep');
  // wait for the user to be ready to start
  await wait_for_input();
  // run it! resume ticks then wait for end conditions
  await run_benchmark();
  // print history.log then shut down the containers
  await post_run_cleanup();
}

run().catch(console.error)
