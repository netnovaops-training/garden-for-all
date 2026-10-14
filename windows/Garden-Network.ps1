# =============================================================================
#  Garden-Network.ps1: this PC's own address, and the computers at home that
#  have the SSH door open, so you can pick the garden computer instead of
#  typing its address. Used by the guided installer (Garden-Setup.ps1) and the
#  menu (Garden-Menu.ps1).
#
#  What it does, exactly: it looks at THIS PC's home network (the one that
#  leads to the router) and knocks once on port 22 of each address in it, for
#  example 192.168.1.1 to 192.168.1.254, waiting at most 1.5 seconds in all.
#  Nothing is sent and nothing logs in: it only asks "is the SSH door there?".
#  Port 22 is where an Ubuntu computer or a Raspberry Pi with the SSH server
#  listens (some NAS boxes too). Then it asks the router for each one's name.
# =============================================================================

# This PC: its name, its address on the home network, and the router's address.
function Get-This-PC {
    $c = Get-NetIPConfiguration -ErrorAction SilentlyContinue |
        Where-Object { $_.IPv4DefaultGateway -and $_.NetAdapter.Status -eq "Up" } | Select-Object -First 1
    if (-not $c) { return $null }
    $ip = $c.IPv4Address | Select-Object -First 1
    return [pscustomobject]@{ Name = $env:COMPUTERNAME; IP = $ip.IPAddress; Prefix = $ip.PrefixLength
                              Router = $c.IPv4DefaultGateway.NextHop }
}

# The computers at home that answer on port 22, each with its name if the router knows it.
# A home network is almost always x.y.z.1 to x.y.z.254; a bigger one is searched in that part only.
function Find-Home-Computers {
    $me = Get-This-PC
    if (-not $me) { return @() }
    $base = ($me.IP -split '\.')[0..2] -join '.'
    $tries = @()
    foreach ($n in 1..254) {
        $a = "$base.$n"
        if ($a -eq $me.IP) { continue }
        $c = New-Object Net.Sockets.TcpClient
        $tries += [pscustomobject]@{ IP = $a; Client = $c; Task = $c.ConnectAsync($a, 22) }
    }
    # All knocks at once; the doors that are not there are given up after 1.5 seconds.
    try { [void][Threading.Tasks.Task]::WaitAll([Threading.Tasks.Task[]]@($tries | ForEach-Object { $_.Task }), 1500) } catch { }
    $found = @()
    foreach ($t in $tries) {
        if ($t.Task.Status -eq "RanToCompletion" -and $t.Client.Connected) { $found += $t.IP }
        $t.Client.Close()
    }
    foreach ($ip in $found) {
        $name = ""
        try { $r = [Net.Dns]::GetHostEntryAsync($ip); if ($r.Wait(1000)) { $name = $r.Result.HostName } } catch { }
        if ($name -eq $ip) { $name = "" }
        [pscustomobject]@{ IP = $ip; Name = $name }
    }
}
