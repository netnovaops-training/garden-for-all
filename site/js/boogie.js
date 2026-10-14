// ─────────────────────────────────────────────────────────────────────────────
// boogie.js — End of Garden Easter Egg 🪴🕺
// Triggered when the user clicks "End of Garden. Boogie?"
// ─────────────────────────────────────────────────────────────────────────────

const GARDEN_EMOJIS  = ['🌿','🌸','🪴','🌻','🌵','🌷','🌲','🌳','🌺','🌾','🍄','🌱','🌼','🌈','🦋','🐝','🐇','🦌','🌙','⭐','💐','🍂','🍁','🌍','☀️'];
const BIRD_EMOJIS    = ['🐦','🦜','🦚','🦩','🦅','🕊️','🦆','🐧','🦉','🦢'];
const DOG_EMOJIS     = ['🐕','🐶','🦮','🐩','🐾'];
const BARKS          = ['Woof!','Bark!','Bork!','WOOF!','arf arf!','BORK!','Awoo!'];
const DISCO_COLORS   = ['#ff6b6b','#ffd93d','#6bcb77','#4d96ff','#ff922b','#cc5de8','#f06595','#66d9e8','#a9e34b','#ff6bff'];

let boogieActive  = false;
let boogieOverlay = null;
let boogieAudio   = null;
let boogieTimers  = [];

// ── Public entry point ────────────────────────────────────────────────────────
export async function startBoogie(imgBase) {
    if (boogieActive) return;
    boogieActive = true;

    // Fetch 10 random plant images from the catalog
    let plantImages = [];
    try {
        const pages = [1, 2, 3];
        const page  = pages[Math.floor(Math.random() * pages.length)];
        const res   = await fetch(`catalog.php?ajax_load=1&page=${page}&_t=${Date.now()}`);
        const data  = await res.json();
        const plants = (data.results || []).filter(p => p.picture);
        const shuffled = plants.sort(() => Math.random() - 0.5).slice(0, 10);
        plantImages = shuffled.map(p => imgBase + p.picture.split(',')[0].trim());
    } catch (_) {}

    _injectCSS();
    _buildOverlay(plantImages, imgBase);
    _playAudio();
    _startEffects(plantImages, imgBase);
}

// ── CSS ───────────────────────────────────────────────────────────────────────
function _injectCSS() {
    if (document.getElementById('boogie-css')) return;
    const s = document.createElement('style');
    s.id = 'boogie-css';
    s.textContent = `
        #boogie-overlay {
            position:fixed; inset:0; z-index:99999;
            background:rgba(0,0,0,0.88);
            display:flex; flex-direction:column;
            align-items:center; justify-content:center;
            overflow:hidden;
        }
        #boogie-title {
            font-size:clamp(2rem,6vw,4rem); font-weight:900;
            color:#fff; text-align:center; z-index:10;
            text-shadow:0 0 30px #ff6bff, 0 0 60px #4d96ff;
            animation:boogiePulse 0.8s ease-in-out infinite alternate;
            letter-spacing:3px; margin-bottom:16px;
        }
        #boogie-stop {
            z-index:10; padding:12px 36px; border-radius:50px;
            background:rgba(0,0,0,0.6); color:#fff; border:2px solid #ff6bff;
            font-size:1.1rem; font-weight:900; cursor:pointer;
            letter-spacing:1px; backdrop-filter:blur(6px);
            transition:all 0.2s; text-shadow:0 0 10px #ff6bff;
        }
        #boogie-stop:hover { background:#ff6bff; color:#000; }

        .boogie-disco-spot {
            position:absolute; border-radius:50%;
            pointer-events:none; mix-blend-mode:screen;
            animation:discoMove var(--dur) var(--delay) ease-in-out infinite alternate;
        }
        .boogie-emoji-rain {
            position:absolute; top:-80px; pointer-events:none;
            font-size:var(--sz); animation:emojiRain var(--dur) var(--delay) linear infinite;
            z-index:5;
        }
        .boogie-plant-img {
            position:absolute; pointer-events:none;
            object-fit:cover; border-radius:50%;
            border:3px solid var(--clr);
            box-shadow:0 0 20px var(--clr), 0 0 40px var(--clr);
            animation:plantSpin var(--sdur) linear infinite;
            z-index:4; opacity:0.9;
            transition:border-color 0.3s;
        }
        .boogie-dog {
            position:absolute; bottom:0; font-size:var(--sz);
            animation:dogWalk var(--dur) var(--delay) linear infinite;
            z-index:6; filter:drop-shadow(0 0 8px #ffd93d);
        }
        .boogie-bark {
            position:absolute; background:#ffd93d; color:#333;
            border-radius:12px; padding:3px 8px; font-size:0.75rem;
            font-weight:900; pointer-events:none; z-index:7;
            animation:barkPop 0.5s ease-out forwards;
        }
        .boogie-bird {
            position:absolute; font-size:var(--sz);
            z-index:6; filter:drop-shadow(0 0 6px #66d9e8);
            line-height:1;
        }

        @keyframes boogiePulse {
            from { text-shadow:0 0 20px #ff6bff,0 0 40px #4d96ff; transform:scale(1); }
            to   { text-shadow:0 0 60px #ffd93d,0 0 100px #6bcb77; transform:scale(1.04); }
        }
        @keyframes discoMove {
            0%   { transform:translate(var(--x1),var(--y1)) scale(1);   opacity:0.4; }
            50%  { opacity:0.85; }
            100% { transform:translate(var(--x2),var(--y2)) scale(1.4); opacity:0.2; }
        }
        @keyframes emojiRain {
            0%   { transform:translateY(-80px) rotate(0deg);   opacity:1; }
            100% { transform:translateY(110vh) rotate(720deg); opacity:0.3; }
        }
        @keyframes plantSpin {
            from { filter:hue-rotate(0deg) drop-shadow(0 0 12px #ff6bff); }
            to   { filter:hue-rotate(360deg) drop-shadow(0 0 20px #ffd93d); }
        }
        @keyframes dogWalk {
            0%   { left:-120px; transform:scaleX(1); }
            49%  { transform:scaleX(1); }
            50%  { transform:scaleX(-1); left:calc(100vw + 120px); }
            51%  { transform:scaleX(-1); }
            100% { left:-120px; transform:scaleX(-1); }
        }
        @keyframes barkPop {
            0%   { opacity:0; transform:scale(0.5) translateY(0); }
            40%  { opacity:1; transform:scale(1.2) translateY(-10px); }
            100% { opacity:0; transform:scale(1) translateY(-30px); }
        }
    `;
    document.head.appendChild(s);
}

// ── Build overlay HTML ────────────────────────────────────────────────────────
function _buildOverlay(plantImages, imgBase) {
    const div = document.createElement('div');
    div.id = 'boogie-overlay';
    div.innerHTML = `
        <div id="boogie-title">🪴 End of Garden — BOOGIE TIME! 🕺</div>
        <button id="boogie-stop">🛑 Stop the Boogie</button>
    `;
    document.body.appendChild(div);
    boogieOverlay = div;

    document.getElementById('boogie-stop').addEventListener('click', stopBoogie);

    // Add disco ball spots
    for (let i = 0; i < 18; i++) _addDiscoSpot(div);

    // Add bouncing plant images
    plantImages.forEach((src, i) => _addPlantImage(div, src, i));
    requestAnimationFrame(_tickBouncers);

    // Add emoji rain
    for (let i = 0; i < 30; i++) _addEmojiRain(div);

    // Add dogs
    for (let i = 0; i < 4; i++) _addDog(div);

    // Add birds — alternate 45° upper-left and 135° upper-right
    for (let i = 0; i < 8; i++) _addBird(div, i);
    requestAnimationFrame(_tickBirds);
}

// ── Disco spots ───────────────────────────────────────────────────────────────
function _addDiscoSpot(parent) {
    const el  = document.createElement('div');
    el.className = 'boogie-disco-spot';
    const size = 40 + Math.random() * 120;
    const color = DISCO_COLORS[Math.floor(Math.random() * DISCO_COLORS.length)];
    const dur  = (3 + Math.random() * 5).toFixed(1);
    const delay = (Math.random() * 4).toFixed(1);
    const x1 = `${Math.random() * 100 - 50}vw`;
    const y1 = `${Math.random() * 100 - 50}vh`;
    const x2 = `${Math.random() * 100 - 50}vw`;
    const y2 = `${Math.random() * 100 - 50}vh`;
    const startX = `${Math.random() * 100}vw`;
    const startY = `${Math.random() * 100}vh`;
    el.style.cssText = `
        width:${size}px; height:${size}px;
        left:${startX}; top:${startY};
        background:radial-gradient(circle, ${color}cc 0%, transparent 70%);
        --dur:${dur}s; --delay:-${delay}s;
        --x1:${x1}; --y1:${y1}; --x2:${x2}; --y2:${y2};
    `;
    parent.appendChild(el);
}

// ── Bouncing plant images (DVD screensaver style) ─────────────────────────────
const _bouncers = [];

function _addPlantImage(parent, src, i) {
    const el   = document.createElement('img');
    el.className = 'boogie-plant-img';
    el.src = src;
    const size  = 70 + Math.random() * 90;
    const color = DISCO_COLORS[i % DISCO_COLORS.length];
    const sdur  = (5 + Math.random() * 8).toFixed(1);
    el.style.cssText = `width:${size}px; height:${size}px; --clr:${color}; --sdur:${sdur}s;`;
    parent.appendChild(el);

    // Start in a random position
    const x  = Math.random() * (window.innerWidth  - size);
    const y  = Math.random() * (window.innerHeight - size);
    // Slow random velocity (0.4–1.2 px/frame)
    const spd = 5 + Math.random() * 4;
    const ang = Math.random() * Math.PI * 2;
    _bouncers.push({
        el, size, color,
        x, y,
        vx: Math.cos(ang) * spd,
        vy: Math.sin(ang) * spd,
        colorIdx: i % DISCO_COLORS.length
    });
}

function _tickBouncers() {
    if (!boogieActive) return;
    const W = window.innerWidth;
    const H = window.innerHeight;
    for (const b of _bouncers) {
        b.x += b.vx;
        b.y += b.vy;
        // Bounce off right/left
        if (b.x + b.size >= W) { b.vx = -Math.abs(b.vx); b.x = W - b.size; _flashColor(b); }
        if (b.x <= 0)           { b.vx =  Math.abs(b.vx); b.x = 0;          _flashColor(b); }
        // Bounce off bottom/top
        if (b.y + b.size >= H) { b.vy = -Math.abs(b.vy); b.y = H - b.size; _flashColor(b); }
        if (b.y <= 0)           { b.vy =  Math.abs(b.vy); b.y = 0;          _flashColor(b); }
        b.el.style.left = b.x + 'px';
        b.el.style.top  = b.y + 'px';
    }
    requestAnimationFrame(_tickBouncers);
}

function _flashColor(b) {
    b.colorIdx = (b.colorIdx + 1) % DISCO_COLORS.length;
    const c = DISCO_COLORS[b.colorIdx];
    b.el.style.setProperty('--clr', c);
    b.el.style.borderColor = c;
    b.el.style.boxShadow = `0 0 20px ${c}, 0 0 40px ${c}`;
}

// ── Emoji rain ────────────────────────────────────────────────────────────────
function _addEmojiRain(parent) {
    const pool = [...GARDEN_EMOJIS, ...BIRD_EMOJIS];
    const el   = document.createElement('div');
    el.className = 'boogie-emoji-rain';
    el.textContent = pool[Math.floor(Math.random() * pool.length)];
    const size  = (1.2 + Math.random() * 2.5).toFixed(1);
    const dur   = (4 + Math.random() * 8).toFixed(1);
    const delay = (Math.random() * 8).toFixed(1);
    const left  = `${Math.random() * 100}vw`;
    el.style.cssText = `left:${left}; --sz:${size}rem; --dur:${dur}s; --delay:-${delay}s;`;
    parent.appendChild(el);
}

// ── Walking barking dogs ──────────────────────────────────────────────────────
function _addDog(parent) {
    const el  = document.createElement('div');
    el.className = 'boogie-dog';
    const dog  = DOG_EMOJIS[Math.floor(Math.random() * DOG_EMOJIS.length)];
    el.textContent = dog;
    const size  = (2.5 + Math.random() * 2).toFixed(1);
    const dur   = (6 + Math.random() * 8).toFixed(1);
    const delay = (Math.random() * 8).toFixed(1);
    const bot   = `${Math.random() * 25}vh`;
    el.style.cssText = `bottom:${bot}; --sz:${size}rem; --dur:${dur}s; --delay:-${delay}s;`;
    parent.appendChild(el);

    // Random barks
    const barkTimer = setInterval(() => {
        if (!boogieActive) { clearInterval(barkTimer); return; }
        const rect = el.getBoundingClientRect();
        if (rect.left < 0 || rect.left > window.innerWidth) return;
        const bark = document.createElement('div');
        bark.className = 'boogie-bark';
        bark.textContent = BARKS[Math.floor(Math.random() * BARKS.length)];
        bark.style.left = `${rect.left + 20}px`;
        bark.style.top  = `${rect.top  - 30}px`;
        document.body.appendChild(bark);
        setTimeout(() => bark.remove(), 600);
    }, 1500 + Math.random() * 2000);
    boogieTimers.push(barkTimer);
}

// ── Flying birds — diagonal 45° and 135° (no spin, no drift) ─────────────────
const _birds = [];

function _addBird(parent, idx) {
    const el = document.createElement('div');
    el.className = 'boogie-bird';
    // Alternate between 🕊️ dove and other birds
    el.textContent = idx % 3 === 0 ? '🕊️' : BIRD_EMOJIS[Math.floor(Math.random() * BIRD_EMOJIS.length)];
    const size = (1.8 + Math.random() * 1.5).toFixed(1);
    el.style.cssText = `--sz:${size}rem;`;
    parent.appendChild(el);

    const spd  = 1.5 + Math.random() * 1.5;
    // Even index → fly upper-left (45°): vx negative, vy negative
    // Odd  index → fly upper-right (135°): vx positive, vy negative
    const goRight = idx % 2 === 1;
    const vx = goRight ?  spd : -spd;
    const vy = -spd; // always upward

    // Start from a random position along the opposite edge
    const x = goRight ? -60 : window.innerWidth  + 60;
    const y = window.innerHeight * (0.2 + Math.random() * 0.6);

    // Flip emoji to face direction of travel
    el.style.transform = goRight ? 'scaleX(1)' : 'scaleX(-1)';

    _birds.push({ el, x, y, vx, vy, goRight });
}

function _tickBirds() {
    if (!boogieActive) return;
    const W = window.innerWidth;
    const H = window.innerHeight;
    for (const b of _birds) {
        b.x += b.vx;
        b.y += b.vy;
        // Reset when off-screen — re-enter from the same starting edge, new random height
        const offScreen = b.goRight ? b.x > W + 80 : b.x < -80;
        const offTop    = b.y < -80;
        if (offScreen || offTop) {
            b.x = b.goRight ? -60 : W + 60;
            b.y = H * (0.2 + Math.random() * 0.6);
        }
        b.el.style.left = b.x + 'px';
        b.el.style.top  = b.y + 'px';
    }
    requestAnimationFrame(_tickBirds);
}

// ── Audio ─────────────────────────────────────────────────────────────────────
function _playAudio() {
    try {
        boogieAudio = new Audio('Boogie.mp3');
        boogieAudio.loop   = false;
        boogieAudio.volume = 0.7;
        boogieAudio.addEventListener('ended', () => stopBoogie());
        boogieAudio.play().catch(() => {
            document.getElementById('boogie-overlay')?.addEventListener('click', () => {
                boogieAudio?.play();
            }, { once: true });
        });
    } catch (_) {}
}

// ── Ongoing effects ───────────────────────────────────────────────────────────
function _startEffects(plantImages, imgBase) {
    // Periodically add new emoji bursts
    const burstTimer = setInterval(() => {
        if (!boogieActive) { clearInterval(burstTimer); return; }
        for (let i = 0; i < 3; i++) _addEmojiRain(boogieOverlay);
        // Remove old ones to avoid DOM bloat
        const rains = boogieOverlay.querySelectorAll('.boogie-emoji-rain');
        if (rains.length > 60) rains[0].remove();
    }, 3000);
    boogieTimers.push(burstTimer);
}

// ── Stop ──────────────────────────────────────────────────────────────────────
export function stopBoogie() {
    boogieActive = false;
    _bouncers.length = 0;
    _birds.length = 0;
    boogieTimers.forEach(t => clearInterval(t));
    boogieTimers = [];
    if (boogieAudio) {
        boogieAudio.pause();
        boogieAudio.currentTime = 0;
        boogieAudio = null;
    }
    // Fade out
    if (boogieOverlay) {
        boogieOverlay.style.transition = 'opacity 0.8s';
        boogieOverlay.style.opacity = '0';
        setTimeout(() => {
            boogieOverlay?.remove();
            boogieOverlay = null;
        }, 850);
    }
}
