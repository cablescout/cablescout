import { Tunnel } from './tunnel'

class Status {
    private curr_tunnel?: Tunnel

    currTunnel(): Tunnel {
        return this.curr_tunnel
    }

    setCurrTunnel(tunnel: Tunnel) {
        this.curr_tunnel = tunnel
    }
}

export const STATUS = new Status()
