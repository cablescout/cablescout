class Status {
  constructor() {
    this._curr_tunnel = null
    this._is_connected = false
  }

  // TODO: Find this out automatically somehow
  currTunnel() {
    return this._curr_tunnel
  }

  // TODO: Find this out automatically somehow
  setCurrTunnel(name) {
    this._curr_tunnel = name
  }

  isConnected() {
    return this._is_connected
  }

  setConnected(is_connected) {
    this._is_connected = is_connected
  }
}

module.exports = {
  STATUS: new Status(),
}
