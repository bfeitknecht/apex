# `infra/` — everything the infra repo needs for apex

**Transient.** This directory exists so the changes can be merged into
`bfeitknecht/infra` in one pass, and it is deleted as part of go-live: once the
files below are in the infra repo, `infra/` has no reason to exist here.

The tree mirrors infra's own layout, and **each file is the complete final
content** of that file in `bfeitknecht/infra`, not a diff — so the merge is
"copy over", and the delta is described per file below for whoever has to
reconcile this against glob and exhib, which touch the same files.

```text
infra/
├── README.md                                          this file
├── sshd_config.example                                alternative: host sshd, not the container
└── kubernetes/
    ├── applications/apex/
    │   ├── kustomization.yaml                         MODIFY (adds 3 resources)
    │   ├── tui-deployment.yaml                        NEW    apex-tui Deployment
    │   ├── tui-service.yaml                           NEW    apex-tui Service :2222
    │   └── tui-tcproute.yaml                          NEW    TCPRoute apex-ssh
    ├── cluster/applications.yaml                      MODIFY (Keel ignore for apex-tui)
    └── infrastructure/
        ├── ingress/traefik/gateway.yaml               MODIFY (apexssh listener on 22, gitssh → 2223)
        ├── ingress/traefik/traefik.yaml               MODIFY (apexssh entrypoint on 22, gitssh → 2223)
        └── monitoring/gatus.yaml                      MODIFY (two endpoints)
```

Baseline: `bfeitknecht/infra@4e7c2e1` ("feat: add blog"). Every file here is
that commit plus the apex change and nothing else.

## What the site actually serves

Unchanged at the HTTP layer — `HTTPRoute apex` (`applications/apex/httproute.yaml`)
still points `0xbf.ch` at `Service apex:80`. What changed is only *inside* that
document root, which is why no HTTP manifest is in this directory:

```text
/                 the page: static HTML from Typst, or the WebAssembly app if
                  the browser can run it (both, at the same URL — no /tui, no
                  /static)
/contact/         a content page of the static site
/age.txt /pgp.txt /cv      plain artifacts, rewritten from /cv to /cv.pdf
/404.html         static site's 404
```

## Port ledger

The three agents land on the same Traefik entrypoints, so this is the contract:

| Port | Owner | Notes |
| --- | --- | --- |
| 80, 443 | traefik | web / websecure |
| **22** | **apex** | `ssh 0xbf.ch`; the router's existing WAN 22 rule keeps working |
| 2222 | devil | LB-only, no WAN rule |
| 2223 | forgejo git-ssh | **moved off 22** — clone URLs change |

If another repo needs a raw TCP port, take the next free one (2224, …) and add
it to both `traefik.yaml` (`ports.<name>`) and `gateway.yaml` (a TCP listener
with the same name and port). Never reuse a port that appears above.

## Per-file delta

### `kubernetes/applications/apex/tui-deployment.yaml` — new

`apex-tui`: the same image as the site (`ghcr.io/bfeitknecht/apex:latest`), with
`command: ["/usr/local/bin/apex-ssh"]` overriding the image's
`static-web-server` entrypoint. Public mode (`APEX_SSH_PUBLIC=1`: any username,
no client key), host key from the `apex-ssh-host-key` Secret mounted at
`/run/secrets`, listening on 2222, TCP probes (the port opens only after the key
loads, so a connect is a truthful signal), Keel annotations on the **Deployment**
metadata (they are invisible on the pod template), `imagePullSecrets:
dockerconfigjson` on the pod template (Keel reads it only from there).

A separate Deployment rather than a sidecar in the site pod: a missing Secret
would leave the pod `Pending` and take the homepage down with it.

### `kubernetes/applications/apex/tui-service.yaml`, `tui-tcproute.yaml` — new

`Service apex-tui` (2222/TCP) and `TCPRoute apex-ssh` parented to the
`traefik-gateway` `apexssh` listener, backend `apex-tui:2222`.

### `kubernetes/applications/apex/kustomization.yaml` — modify

Adds the three files above to `resources`. The `ssh-host-key.sops.yaml` entry
stays commented until the Secret exists (create it first, below).

### `kubernetes/cluster/applications.yaml` — modify

`spec.ignore` for the `apex` Kustomization gains a second entry so Flux stops
drifting `Deployment/apex-tui`'s image back to the manifest while Keel tracks
`:latest`. Both Deployments need it.

### `kubernetes/infrastructure/ingress/traefik/traefik.yaml` — modify

- **new entrypoint `apexssh`**: `port: 22`, `exposedPort: 22`, `hostPort: 22`.
- **`gitssh` moves `22 → 2223`** (`port`, `exposedPort`, `hostPort`).

`hostPort` on both is deliberate: the Zyxel router will not forward to the
LoadBalancer IP, so traffic arrives on the node and Traefik distinguishes by
port. Because `apexssh` takes 22, the existing WAN 22 → `10.10.20.10:22` rule
now reaches the portfolio instead of git.

### `kubernetes/infrastructure/ingress/traefik/gateway.yaml` — modify

- **new TCP listener `apexssh` on 22** (matches the entrypoint name).
- **`gitssh` listener `22 → 2223`**, so `forgejo`'s existing TCPRoute (which
  selects `sectionName: gitssh`) keeps working against the new port.

### `kubernetes/infrastructure/monitoring/gatus.yaml` — modify

Two endpoints appended to the config ConfigMap: `apex` (HTTP 200 on
`apex.apex.svc.cluster.local`) and `apex-ssh` (TCP connect to
`apex-tui.apex.svc.cluster.local:2222`).

## Reconciling with glob and exhib

The three repos arrive together. Expect contention exactly here:

| File | Contention | How to merge |
| --- | --- | --- |
| `applications/apex/kustomization.yaml` | none (apex-owned) | copy |
| `cluster/applications.yaml` | glob/exhib add their own `ignore` blocks to *their* Kustomizations | keep every block; they are per-`target.name` |
| `infrastructure/ingress/traefik/traefik.yaml` | both may add entrypoints | keep all named entrypoints; ports from the ledger |
| `infrastructure/ingress/traefik/gateway.yaml` | same, plus listeners | one listener per entrypoint, same `name`, same `port` |
| `infrastructure/monitoring/gatus.yaml` | both add endpoints | append-only list under `endpoints:` |
| `AGENTS.md`, `README.md` (infra root) | each agent edits the same prose | merge the sentences; keep one line per public host |

Rule of thumb: everything except `gitssh` in this directory is *additive*, so a
conflict means "keep both". `gitssh 22 → 2223` is the one destructive change.

### Prose in the infra repo (not copied as whole files)

Two files carry one sentence about apex each. They are prose shared with the
other agents, so they are given as exact edits instead of whole files:

`README.md`, "### Applications", the first bullet:

```diff
-- **Public:** apex site, glob blog (`blog.0xbf.ch`) (Keel-polled `:latest`)
+- **Public:** apex site (`0xbf.ch`: the page and the WebAssembly app at the same URL) and its SSH portfolio on **22** (`ssh 0xbf.ch`), glob blog (`blog.0xbf.ch`) (Keel-polled `:latest`)
```

and, in the same file's tree sketch, one line's comment:

```diff
-        ├── apex/                   # Public site (Keel-polled :latest)
+        ├── apex/                   # Public site + SSH TUI (Keel-polled :latest)
```

`AGENTS.md`, "Tier 2 — dual-config apps", append to the end of that bullet:

```text
The same image also ships the SSH frontend (`/usr/local/bin/apex-ssh`): a **separate** `apex-tui` Deployment in `applications/apex/` (own Service `apex-tui:2222`, `APEX_SSH_PUBLIC=1`, no exec/SFTP/forwarding) so a broken host key or a crashed session loop can never take `0xbf.ch` down. It is reached by `TCPRoute apex-ssh` on the dedicated `apexssh` Gateway listener / Traefik entrypoint on **22** (so `ssh 0xbf.ch` just works); forgejo git-ssh moved to 2223 (clone URLs become `ssh://git@git.0xbf.ch:2223/...`), the dev box keeps 2222. Its identity is the SOPS Secret `apex-ssh-host-key` (`ssh_host_ed25519_key`): replace it and every client sees a changed fingerprint, so back it up with the other secrets.
```

## Doing it

1. **Host identity** (once; it *is* the server identity, back it up):

   ```sh
   ssh-keygen -t ed25519 -N '' -f /tmp/apex_ssh_host_ed25519_key
   ssh-keygen -lf /tmp/apex_ssh_host_ed25519_key.pub   # record the fingerprint
   ```

2. **Encrypt it into the repo** (`.sops.yaml` already matches `*.sops.yaml`):

   ```sh
   cd infra
   kubectl create secret generic apex-ssh-host-key -n apex \
     --from-file=ssh_host_ed25519_key=/tmp/apex_ssh_host_ed25519_key \
     --dry-run=client -o yaml \
     | sops --encrypt --input-type yaml /dev/stdin \
     > kubernetes/applications/apex/ssh-host-key.sops.yaml
   shred -u /tmp/apex_ssh_host_ed25519_key
   ```

   Key name inside the Secret: `ssh_host_ed25519_key` (that is what
   `APEX_SSH_HOST_KEY` points at).

3. **Copy this tree over**, uncomment `ssh-host-key.sops.yaml` in
   `applications/apex/kustomization.yaml`, commit, push.

   These are whole files, not a patch — reconcile first (previous section),
   then copy:

   ```sh
   cp -R /path/to/apex/infra/kubernetes/. kubernetes/
   git status          # eight paths, three of them new
   ```

4. **Reconcile** (Traefik restarts: `protocol: TCP`/`Recreate` and hostPorts,
   so expect ~10 s of downtime for every `*.0xbf.ch` host, not just apex):

   ```sh
   flux reconcile kustomization traefik --with-source
   flux reconcile kustomization apex --with-source
   ```

5. **Router** (Zyxel): add WAN **2223/tcp → 10.10.20.10:2223** (git). WAN 22
   already points at the Pi. No DNS change — the wildcard covers everything.

6. **Clients**: publish the new git remote
   `ssh://git@git.0xbf.ch:2223/<owner>/<repo>.git` wherever it is documented;
   existing checkouts keep a stale `origin` and fail with "connection refused".

## Verify

```sh
kubectl -n apex get pods,svc,tcproute
flux get kustomizations apex traefik
ssh 0xbf.ch                             # no -p; fingerprint must match step 1
git clone ssh://git@git.0xbf.ch:2223/bfeitknecht/infra.git /tmp/check
curl -sI https://0xbf.ch | head -1      # 200, document root
```

Gatus shows `apex` and `apex-ssh` under `status.internal`. From the LAN,
`ssh 10.10.20.10` skips the router and tests the same path.

## The SSH frontend, in short

`ssh 0xbf.ch` serves the portfolio, not a shell: no exec, no SFTP, no
forwarding, PTY required, 32 sessions, 15 min idle timeout, terminal clamped to
20–240 × 8–120 cells. `APEX_SSH_PUBLIC=1` accepts **any** username and needs no
client key, which is the point (strangers should not have to know a user name).
To lock it down instead: `APEX_SSH_PUBLIC=0` and `APEX_SSH_AUTHORIZED_KEYS` at a
mounted allowlist — the process refuses to start if an allowlist is set without
picking a mode explicitly.

If you would rather not ship an SSH server, `infra/sshd_config.example` is the
alternative (host sshd, `ForceCommand`), at the cost of a real system account
and no image-rolled updates.

## After the merge

Delete `infra/` from the apex repo and remove the pointer in `docs/go-live.md`.
`deploy/` is gone already; nothing else in the repo refers to these files.
