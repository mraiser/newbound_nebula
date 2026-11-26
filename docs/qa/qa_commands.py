#!/usr/bin/env python3
"""QA for the nebula app's commands (Phases 1-3), run against a SCRATCH instance.

    python3 -I docs/qa/qa_commands.py <instance-folder> <http-port>

Logs in as admin (the password is read from <instance>/users/admin.properties
and never printed), creates a throw-away network `t-qa`, exercises every
Phase 1-3 behaviour over HTTP the way the page does, then deletes `t-qa`.
Never point it at a live instance.
"""
import json, os, sys, shutil, socket, subprocess, urllib.request, urllib.parse, base64, hashlib, tempfile, time

inst, port = sys.argv[1], sys.argv[2]
base = f'http://localhost:{port}'
nets = os.path.join(inst, 'runtime/nebula/networks')
certbin = os.path.join(inst, 'runtime/nebula/bin/nebula-cert')
engine = os.path.join(inst, 'runtime/nebula/bin/nebula')
results = []

def check(name, ok, detail=''):
    results.append((name, bool(ok), detail))
    print(('PASS ' if ok else 'FAIL ') + name + (f'  [{detail}]' if detail and not ok else ''))

def post(path, body, sid=None):
    req = urllib.request.Request(base + path, data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'})
    if sid: req.add_header('Cookie', 'sessionid=' + sid)
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.loads(r.read().decode())

def cmd(_cmd, sid, **params):
    r = post('/nebula/' + _cmd, params, sid)
    return r.get('data') if r.get('status') == 'ok' else r

def cert_json(path):
    return json.loads(subprocess.run([certbin, 'print', '-json', '-path', path], capture_output=True, text=True).stdout)

def nebula_test(cfg):
    return subprocess.run([engine, '-test', '-config', cfg], capture_output=True).returncode == 0

# ---- 0. log in (password never printed) ----
pw = [l.split('=', 1)[1].strip() for l in open(os.path.join(inst, 'users/admin.properties')) if l.startswith('password=')][0]
login = urllib.request.urlopen(urllib.request.Request(base + '/app/login', data=urllib.parse.urlencode({'user': 'admin', 'pass': pw}).encode()), timeout=30)
sid = json.loads(login.read().decode())['sessionid']; del pw

# ---- 1. every command refuses without a login ----
cmds = ['add_member', 'add_phone', 'build_config', 'create_network', 'endpoints', 'info', 'init', 'install_release', 'install_service',
        'join_network', 'members', 'port_status', 'releases', 'remove_member', 'restart_service', 'save_config', 'set_boot',
        'set_firewall', 'start', 'start_service', 'stop', 'stop_service', 'uninstall_service', 'update_blocklist', 'update_hosts']
refused = [c for c in cmds if (lambda r: r.get('status') == 'err' and 'UNAUTHORIZED' in r.get('msg', ''))(post('/nebula/' + c, {}))]
check('logged out: all %d nebula commands refuse (UNAUTHORIZED)' % len(cmds), len(refused) == len(cmds), 'not refused: ' + str(sorted(set(cmds) - set(refused))))

# ---- 2. info (#2 tun, #5 expiry, #9 http_address) ----
info = cmd('info', sid)
t = info.get('tun', {})
check('info: tun block has os/arch/how', all(k in t for k in ('os', 'arch', 'how')), str(t)[:120])
check('info: tun.ready reported on Linux (getcap present)', t.get('os') != 'linux' or 'ready' in t)
check('info: http_address reported', 'http_address' in info)

# ---- 3. port_status (#8) ----
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.bind(('0.0.0.0', 0)); busy = s.getsockname()[1]
r = cmd('port_status', sid, port=str(busy)); check('port_status: a bound port reads bound', r.get('bound') is True, str(r))
s.close()
r = cmd('port_status', sid, port=str(busy)); check('port_status: the same port reads free once released', r.get('bound') is False, str(r))
r = cmd('port_status', sid, port='abc'); check('port_status: a non-number is refused in words', r.get('status') == 'err', str(r))

# ---- 4. create_network (#5 10-year CA, §5 member firewall) ----
shutil.rmtree(os.path.join(nets, 't-qa'), ignore_errors=True)
r = cmd('create_network', sid, name='t-qa', subnet='192.168.248.X/24', port='4260')
home = os.path.join(nets, 't-qa')
ca = cert_json(os.path.join(home, 'ca.crt'))['details']
years = (time.mktime(time.strptime(ca['notAfter'][:19], '%Y-%m-%dT%H:%M:%S')) - time.time()) / (365.25 * 86400)
check('create_network: CA lasts ~10 years', 9.9 < years < 10.01, '%.2f years' % years)
check('create_network: this host cert in group member', cert_json(os.path.join(home, 'host.crt'))['details']['groups'] == ['member'])
conn = json.load(open(os.path.join(home, 'connection.json')))
check('create_network: firewall=member recorded', conn.get('firewall') == 'member')
y = open(os.path.join(home, 'config.yml')).read()
check('create_network: inbound rules are groups [member]', y.count('groups: [member]') == 2 and 'host: any\n' not in y.split('inbound:')[1])
check('create_network: nebula -test accepts the config', nebula_test(os.path.join(home, 'config.yml')))

# ---- 5. add_phone refusals ----
r = cmd('add_phone', sid, servicename='t-qa', device_name='QA phone', groups='', pubkey='')
check('add_phone: no lighthouse -> refused with reason no_lighthouse', r.get('reason') == 'no_lighthouse', str(r))
conn.pop('yaml', None); conn['am_lighthouse'] = True; conn['public_address'] = '203.0.113.20'
cmd('save_config', sid, servicename='t-qa', config=conn)
r = cmd('add_phone', sid, servicename='t-qa', device_name='', groups='', pubkey='')
check('add_phone: empty name refused', r.get('reason') == 'bad_name', str(r))
r = cmd('add_phone', sid, servicename='t-qa', device_name='QA phone', groups='bad group!', pubkey='')
check('add_phone: bad group refused', r.get('reason') == 'bad_groups', str(r))

# ---- 6. Mode A ----
r = cmd('add_phone', sid, servicename='t-qa', device_name='QA phone A', groups='family', pubkey='')
j = r.get('join', {})
check('Mode A: kind join with key', j.get('kind') == 'join' and 'PRIVATE KEY' in j.get('key', ''))
check('Mode A: public address gets the network port', j.get('lighthouses') == [{'addrs': ['203.0.113.20:4260'], 'nebula_ip': '192.168.248.1'}], str(j.get('lighthouses')))
check('Mode A: groups member,phone,family', j.get('groups') == ['member', 'phone', 'family'])
check('Mode A: expires = created + 600', j.get('expires', 0) - j.get('created', 0) == 600)
check('Mode A: owner + newbound_url filled', j.get('owner', {}).get('peer_id') and j.get('newbound_url', '').startswith('http://192.168.248.1:'))
mdir = os.path.join(home, 'members', r.get('member_id', '?'))
check('Mode A: member folder holds only host.crt + info.json', sorted(os.listdir(mdir)) == ['host.crt', 'info.json'] if os.path.isdir(mdir) else False)
check('Mode A: no temp folder left', not [d for d in os.listdir(home) if d.startswith('.add-phone')])
tmp = tempfile.mkdtemp()
open(tmp + '/ca.crt', 'w').write(j['ca']); open(tmp + '/c.crt', 'w').write(j['cert'])
check('Mode A: cert verifies against the CA', subprocess.run([certbin, 'verify', '-ca', tmp + '/ca.crt', '-crt', tmp + '/c.crt'], capture_output=True).returncode == 0)
check('Mode A: ca_fingerprint matches', cert_json(tmp + '/ca.crt')['fingerprint'] == j['ca_fingerprint'])

# ---- 7. Mode B2 ----
subprocess.run([certbin, 'keygen', '-out-key', tmp + '/p.key', '-out-pub', tmp + '/p.pub'], check=True)
pub = open(tmp + '/p.pub').read()
r = cmd('add_phone', sid, servicename='t-qa', device_name='QA phone B2', groups='', pubkey=pub)
j2 = r.get('join', {})
raw = base64.b64decode(''.join(l for l in pub.splitlines() if not l.startswith('-----')))
check('B2: kind cert, no key', j2.get('kind') == 'cert' and 'key' not in j2)
check('B2: pubkey_fingerprint = SHA-256 of the raw key', j2.get('pubkey_fingerprint') == hashlib.sha256(raw).hexdigest())
open(tmp + '/b2.crt', 'w').write(j2.get('cert', ''))
check('B2: cert holds the phone public key', cert_json(tmp + '/b2.crt')['details']['publicKey'] == raw.hex())
r = cmd('add_phone', sid, servicename='t-qa', device_name='x', groups='', pubkey='-----BEGIN NEBULA X25519 PUBLIC KEY-----\nAAAA\n-----END NEBULA X25519 PUBLIC KEY-----')
check('B2: damaged key refused in words', r.get('reason') == 'bad_pubkey', str(r))

# ---- 8. members ----
m = cmd('members', sid, servicename='t-qa')
phones = [v for v in m.values() if v.get('kind') == 'phone']
check('members: both phones listed with added/not_after/fingerprint', len(phones) == 2 and all(p.get('added') and p.get('not_after') and p.get('fingerprint') for p in phones))
check('members: modes recorded (a, b2)', sorted(p.get('mode') for p in phones) == ['a', 'b2'])

# ---- 9. add_member on a member-firewall network ----
r = cmd('add_member', sid, servicename='t-qa', peer='qa-peer-uuid', ipaddress='192.168.248.9/24', groups='family')
check('add_member: member group added on member-firewall network', r.get('groups') == 'member,family', str(r.get('groups')))

# ---- 10. Remove -> blocklist ----
pid = [k for k, v in m.items() if v.get('name') == 'QA phone A'][0]
fp = m[pid]['fingerprint']
r = cmd('remove_member', sid, servicename='t-qa', peer=pid)
check('remove_member: ok with the fingerprint', r.get('status') == 'ok' and r.get('fingerprint') == fp, str(r)[:150])
check('remove_member: folder deleted', not os.path.exists(os.path.join(home, 'members', pid)))
y = open(os.path.join(home, 'config.yml')).read()
check('remove_member: config.yml blocklists it', f'"{fp}"' in y.split('blocklist:')[1].split('static_host_map')[0] if 'blocklist:' in y else False)
check('remove_member: nebula -test accepts the blocklist', nebula_test(os.path.join(home, 'config.yml')))
r = cmd('remove_member', sid, servicename='t-qa', peer='no-such-member')
check('remove_member: unknown member refused', r.get('status') == 'err')
r = cmd('update_blocklist', sid, servicename='t-qa', blocklist={'nothex': {}})
check('update_blocklist: refuses a non-fingerprint', r.get('status') == 'err')
r = cmd('set_firewall', sid, servicename='t-qa', mode='everyone')
check('set_firewall: refuses an unknown mode', r.get('status') == 'err')

# ---- 11. old networks untouched ----
old = [n for n in os.listdir(nets) if n in ('t-before', 't-after', 't-soon') and os.path.exists(os.path.join(nets, n, 'config.yml'))]
same = True
for n in old:
    before = open(os.path.join(nets, n, 'config.yml')).read()
    cmd('build_config', sid, servicename=n)
    same = same and open(os.path.join(nets, n, 'config.yml')).read() == before
check('older networks re-render byte-identical (%s)' % ','.join(old), same and old)

# ---- 12. Run now / Stop (supervised) ----
r = cmd('start', sid, servicename='t-qa')
ok = r.get('status') == 'ok'
if not ok and r.get('reason') == 'tun_permission':
    check('start: tun permission failure explained (reason, fix, details)', r.get('tun', {}).get('how') and r.get('details'))
else:
    check('start: runs (engine has its permission)', ok, str(r)[:150])
    st = cmd('info', sid)
    run = [n for n in st.get('networks', []) if n['name'] == 't-qa'][0]
    check('info: t-qa running (supervised)', run.get('supervised') is True)
    check('stop: stops it', cmd('stop', sid, servicename='t-qa').get('was_running') is True)

# ---- cleanup ----
shutil.rmtree(home, ignore_errors=True); shutil.rmtree(tmp, ignore_errors=True)
fails = [r for r in results if not r[1]]
print('\n%d checks, %d failed' % (len(results), len(fails)))
sys.exit(1 if fails else 0)
