import * as path from 'path'
import * as grpc from '@grpc/grpc-js'
import * as protoLoader from '@grpc/proto-loader'
import { dialog } from 'electron'
import log from 'electron-log'
import { ProtoGrpcType } from '../proto-gen/daemon'
import { DaemonClient } from '../proto-gen/daemon_api/Daemon'
import { StatusRequest } from '../proto-gen/daemon_api/StatusRequest'
import { StatusResponse } from '../proto-gen/daemon_api/StatusResponse'
import { StartConnectTunnelRequest } from '../proto-gen/daemon_api/StartConnectTunnelRequest'
import { StartConnectTunnelResponse } from '../proto-gen/daemon_api/StartConnectTunnelResponse'
import { FinishConnectTunnelRequest } from '../proto-gen/daemon_api/FinishConnectTunnelRequest'
import { FinishConnectTunnelResponse } from '../proto-gen/daemon_api/FinishConnectTunnelResponse'
import { DisconnectTunnelRequest } from '../proto-gen/daemon_api/DisconnectTunnelRequest'
import { DisconnectTunnelResponse } from '../proto-gen/daemon_api/DisconnectTunnelResponse'
import { updateTray } from './tray'
import { oauthLogin } from './login'

const PROTO_DIR = path.join(__dirname, '..', 'proto')

const host = '127.0.0.1:51889'
const packageDefinition = protoLoader.loadSync(path.join(PROTO_DIR, 'daemon.proto'))
const proto = grpc.loadPackageDefinition(packageDefinition) as unknown as ProtoGrpcType

function getClient(): Promise<DaemonClient> {
    return new Promise((resolve, reject) => {
        log.debug('[grpc] Creating client')
        const client = new proto.daemon_api.Daemon(host, grpc.credentials.createInsecure())
        const deadline = new Date()
        deadline.setSeconds(deadline.getSeconds() + 5)
        client.waitForReady(deadline, (error?: Error) => {
            if (error) {
                reject(error)
            } else {
                log.debug('[grpc] client ready')
                resolve(client)
            }
        })
    })
}

export async function getStatus(): Promise<StatusResponse> {
    const client = await getClient()
    return await new Promise((resolve, reject) => {
        log.debug('[grpc] Sending StatusRequest')
        client.getStatus(
            {} as StatusRequest,
            (error?: grpc.ServiceError, status?: StatusResponse) => {
                if (error) {
                    reject(error)
                    return
                }
                if (status) {
                    log.debug(`[grpc] Got StatusResponse: ${JSON.stringify(status)}`)
                    resolve(status)
                } else {
                    reject(new Error('getStatus returned empty response'))
                }
            }
        )
    })
}

async function startConnectTunnel(name: string): Promise<StartConnectTunnelResponse> {
    const client = await getClient()
    return await new Promise((resolve, reject) => {
        log.debug('[grpc] Sending StartConnectTunnelRequest')
        client.startConnectTunnel(
            {
                name,
            } as StartConnectTunnelRequest,
            (error?: grpc.ServiceError, response?: StartConnectTunnelResponse) => {
                if (error) {
                    reject(error)
                    return
                }
                if (response) {
                    log.debug(`[grpc] Got StartConnectTunnelResponse: ${JSON.stringify(response)}`)
                    resolve(response)
                } else {
                    reject(new Error('startConnectTunnel returned empty response'))
                }
            }
        )
    })
}

async function finishConnectTunnel(auth_code: string): Promise<FinishConnectTunnelResponse> {
    const client = await getClient()
    return await new Promise((resolve, reject) => {
        log.debug('[grpc] Sending FinishConnectTunnelRequest')

        client.finishConnectTunnel(
            {
                authCode: auth_code,
            } as FinishConnectTunnelRequest,
            (error?: grpc.ServiceError, response?: FinishConnectTunnelResponse) => {
                if (error) {
                    reject(error)
                    return
                }
                if (response) {
                    log.debug(`[grpc] Got FinishConnectTunnelResponse: ${JSON.stringify(response)}`)
                    resolve(response)
                } else {
                    reject(new Error('finishConnectTunnel returned empty response'))
                }
            }
        )
    })
}

export async function connectTunnel(name: string): Promise<void> {
    try {
        log.info('[grpc] Connecting tunnel')
        const start_res = await startConnectTunnel(name)
        if (!start_res.authUrl || !start_res.finishUrl) {
            throw new Error('startConnectTunnel returned null fields')
        }
        await updateTray()
        const auth_code = await oauthLogin(start_res.authUrl, start_res.finishUrl)
        await finishConnectTunnel(auth_code)
    } catch (err) {
        log.error(`[client] error: ${err}`)
        try {
            await disconnectTunnel()
        } catch (err) {
            log.warn(`[client] While trying to disconnect client after error: ${err}`)
        }
        dialog.showErrorBox(`Error connecting to ${name}`, `${err}`)
    }
    await updateTray()
}

export async function disconnectTunnel(): Promise<DisconnectTunnelResponse> {
    const client = await getClient()
    return await new Promise((resolve, reject) => {
        log.debug('[grpc] Sending DisconnectTunnelRequest')
        client.disconnectTunnel(
            {} as DisconnectTunnelRequest,
            (error?: grpc.ServiceError, response?: DisconnectTunnelResponse) => {
                if (error) {
                    reject(error)
                    return
                }
                if (response) {
                    log.debug(`[grpc] Got DisconnectTunnelResponse: ${JSON.stringify(response)}`)
                    resolve(response)
                } else {
                    reject(new Error('disconnectTunnel returned empty response'))
                }
            }
        )
    })
}
