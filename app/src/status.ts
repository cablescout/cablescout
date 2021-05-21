class Status {
    private curr_tunnel?: string
    private is_connected = false

    // TODO: Find this out automatically somehow
    currTunnel(): string {
        return this.curr_tunnel
    }

    // TODO: Find this out automatically somehow
    setCurrTunnel(name?: string) {
        this.curr_tunnel = name
    }

    isConnected(): boolean {
        return this.is_connected
    }

    setConnected(is_connected: boolean) {
        this.is_connected = is_connected
    }
}

export const STATUS = new Status()
