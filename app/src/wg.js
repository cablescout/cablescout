const path = require('path')
const childProcess = require('child_process')
const sudo = require('sudo-prompt')

const WG_CONFIG_DIR = '/etc/wireguard'

async function exec(command, as_root) {
  const exec_func = as_root ? ((cmd, cb) => sudo.exec(cmd, { name: 'Cablescout' }, cb)) : childProcess.exec
  return new Promise((resolve, reject) => {
    exec_func(command, (error, stdout, stderr) => {
      if (error) {
        reject(`Error running command: ${error}:\n${stdout}\n${stderr}`)
      } else {
        resolve(stdout.trim())
      }
    })
  })
}

async function wgKeyPair() {
  console.debug('Creating key pair')
  const private_key = await exec('wg genkey')
  const public_key = await exec(`echo ${private_key} | wg pubkey`)
  console.debug(`Public key: ${public_key}`)
  return { public_key, private_key }
}

async function wgQuickUp(name, config_ini) {
  console.debug(`Running wg-quick up ${name}`)
  const result = await exec(`bash -ec '
(wg-quick down ${name} || true)
mkdir -p ${WG_CONFIG_DIR}
cat <<EOF >${WG_CONFIG_DIR}/${name}.conf
${config_ini}
EOF
wg-quick up ${name}
'`, true)
  console.debug(`Result: ${result.toString('utf8')}`)
}

async function wgQuickDown(name) {
  console.debug(`Running wg-quick down ${name}`)
  const result = await exec(`wg-quick down ${name}`, true)
  console.debug(`Result: ${result.toString('utf8')}`)
}

module.exports = {
  wgKeyPair,
  wgQuickUp,
  wgQuickDown,
}
