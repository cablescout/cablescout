import axios from 'axios'
import log from 'electron-log'
import { wgKeyPair, wgQuickUp, wgQuickDown, WgQuickConfig } from './wg'
import { oauthLogin } from './login'

export interface TunnelConfig {
    endpoint: string,
}

export class Tunnel {
    private is_connected = false

    constructor(public name: string, public config: TunnelConfig) {
    }

    private startApiUrl(): string {
        const url = new URL('/api/v1/login/start', this.config.endpoint)
        return url.toString()
    }

    private finishApiUrl(): string {
        const url = new URL('/api/v1/login/finish', this.config.endpoint)
        return url.toString()
    }

    private finishUrl(): string {
        const url = new URL('/finish', this.config.endpoint)
        return url.toString()
    }

    async login(): Promise<WgQuickConfig> {
        log.info(`[tunnel:${this.name}:login] Logging in`)

        const key_pair = await wgKeyPair()
        log.debug(`[tunnel:${this.name}:login] Public key: ${key_pair.public_key}`)

        const start_api_url = this.startApiUrl()
        log.debug(`[tunnel:${this.name}:login] Calling ${start_api_url}`)
        const start_res = await axios.post(start_api_url, {
            client_public_key: key_pair.public_key,
        })

        const auth_code = await oauthLogin(start_res.data.auth_url, this.finishUrl())

        const finish_api_url = this.finishApiUrl()
        log.debug(`[tunnel:${this.name}:login] Calling ${finish_api_url}`)
        const finish_res = await axios.post(finish_api_url, {
            login_token: start_res.data.login_token,
            auth_code,
        })

        const { session_ends_at } = finish_res.data

        return {
            Interface: {
                PrivateKey: key_pair.private_key,
                ...finish_res.data.interface,
            },
            Peer: finish_res.data.peer,
        }
    }

    async connect(): Promise<void> {
        log.info(`[tunnel:${this.name}:connect] Connecting`)
        const config = await this.login()
        await wgQuickUp(this.name, config)
        this.is_connected = true
    }

    async disconnect(): Promise<void> {
        log.info(`[tunnel:${this.name}:disconnect] Disconnecting`)
        await wgQuickDown(this.name)
        this.is_connected = false
    }

    isConnected(): boolean {
        return this.is_connected
    }
}
