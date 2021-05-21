import * as childProcess from 'child_process'
import log from 'electron-log'
import * as sudo from 'sudo-prompt'
import { to_ini } from './stupid-ini'

const WG_CONFIG_DIR = '/etc/wireguard'

export interface WgQuickInterface {
    PrivateKey: string;
    Address: string;
    Dns?: string;
    Mtu?: number;
    ListenPort?: number;
}

export interface WgQuickPeer {
    PublicKey: string;
    AllowedIps: string;
    Endpoint: string;
    PersistentKeepalive?: number;
}

export interface WgQuickConfig {
    Interface: WgQuickInterface;
    Peer: WgQuickPeer;
}

async function exec(command: string, as_root = false): Promise<string> {
    return new Promise((resolve, reject) => {
        const callback = (error: Error, stdout: string | Buffer, stderr: string | Buffer) => {
            if (error) {
                reject(`Error running command: ${error}:\n${stdout}\n${stderr}`)
            } else if (stdout instanceof Buffer) {
                resolve(stdout.toString().trim())
            } else {
                resolve(stdout.trim())
            }
        }
        if (as_root) {
            sudo.exec(command, { name: 'Cablescout' }, callback)
        } else {
            childProcess.exec(command, callback)
        }
    })
}

export interface WgKeyPair {
    public_key: string,
    private_key: string,
}

export async function wgKeyPair(): Promise<WgKeyPair> {
    log.debug('[wg] Creating key pair')
    const private_key = await exec('wg genkey')
    const public_key = await exec(`echo ${private_key} | wg pubkey`)
    return { public_key, private_key }
}

export async function wgQuickUp(name: string, config: WgQuickConfig): Promise<void> {
    log.debug(`[wg] Running wg-quick up ${name}`)
    const config_ini = to_ini(config)
    const result = await exec(`bash -ec '
(wg-quick down ${name} || true)
mkdir -p ${WG_CONFIG_DIR}
cat <<EOF >${WG_CONFIG_DIR}/${name}.conf
${config_ini}
EOF
wg-quick up ${name}
'`, true)
    log.debug(`[wg] Result: ${result.toString()}`)
}

export async function wgQuickDown(name: string): Promise<void> {
    log.debug(`[wg] Running wg-quick down ${name}`)
    const result = await exec(`wg-quick down ${name}`, true)
    log.debug(`[wg] Result: ${result.toString()}`)
}
