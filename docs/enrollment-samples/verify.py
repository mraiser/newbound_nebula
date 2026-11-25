"""Independent check of a join code (ENROLLMENT.md §1-2): decode, compare,
verify the cert chain with nebula-cert, and check the key matches the cert."""
import sys, json, zlib, base64, subprocess, tempfile, os
d, certbin = sys.argv[1], sys.argv[2]
url = open(d + '/join.url').read().strip()
assert url.startswith('newbound://join?v=1&d='), 'URL form'
payload = url.split('&d=', 1)[1]
raw = zlib.decompress(base64.urlsafe_b64decode(payload + '=' * (-len(payload) % 4)), wbits=-15)
j = json.loads(raw)
assert j == json.load(open(d + '/join.decoded.json')), 'decoded JSON differs'
assert open(d + '/join.nbjoin').read().strip() == url and open(d + '/join.payload').read().strip() == payload
for k in ['v','kind','network','ca_fingerprint','owner','newbound_url','lighthouses','created','device_name','ip','groups','ca','cert','key','expires']:
    assert k in j, 'missing ' + k
assert j['v'] == 1 and j['kind'] == 'join' and j['lighthouses'] and j['lighthouses'][0]['addrs']
assert j['expires'] - j['created'] == 600 and j['groups'][:2] == ['member', 'phone']
t = tempfile.mkdtemp()
open(t + '/ca.crt', 'w').write(j['ca']); open(t + '/host.crt', 'w').write(j['cert'])
v = subprocess.run([certbin, 'verify', '-ca', t + '/ca.crt', '-crt', t + '/host.crt'], capture_output=True, text=True)
assert v.returncode == 0, 'cert does not verify: ' + v.stderr
ca = json.loads(subprocess.run([certbin, 'print', '-json', '-path', t + '/ca.crt'], capture_output=True, text=True).stdout)
crt = json.loads(subprocess.run([certbin, 'print', '-json', '-path', t + '/host.crt'], capture_output=True, text=True).stdout)
assert ca['fingerprint'] == j['ca_fingerprint'], 'ca_fingerprint'
assert crt['details']['ips'] == [j['ip']] and crt['details']['name'] == j['device_name'] and crt['details']['groups'] == j['groups']
# X25519 (RFC 7748) public key from the private key, pure Python
P = 2**255 - 19
def x25519(k, u=9):
    k = bytearray(k); k[0] &= 248; k[31] &= 127; k[31] |= 64; k = int.from_bytes(k, 'little')
    x1, x2, z2, x3, z3, swap = u, 1, 0, u, 1, 0
    for t_ in reversed(range(255)):
        kt = (k >> t_) & 1; swap ^= kt
        if swap: x2, x3, z2, z3 = x3, x2, z3, z2
        swap = kt
        A = x2 + z2; AA = A * A; B = x2 - z2; BB = B * B; E = AA - BB; C = x3 + z3; D = x3 - z3
        DA = D * A; CB = C * B
        x3 = (DA + CB) ** 2 % P; z3 = x1 * (DA - CB) ** 2 % P; x2 = AA * BB % P; z2 = E * (AA + 121665 * E) % P
    if swap: x2, x3, z2, z3 = x3, x2, z3, z2
    return (x2 * pow(z2, P - 2, P) % P).to_bytes(32, 'little')
body = ''.join(l for l in j['key'].strip().splitlines() if not l.startswith('-----'))
assert 'NEBULA X25519 PRIVATE KEY' in j['key']
priv = base64.b64decode(body); assert len(priv) == 32
assert x25519(priv).hex() == crt['details']['publicKey'], 'key does not match the cert'
print('OK: decodes, cert verifies against CA, fingerprint matches, key matches cert;', len(url), 'chars,', j['ip'], j['groups'])
