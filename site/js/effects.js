// js/effects.js

// 1. ==========================================
// FIREWORK PARTICLES
// ==========================================
export function launchFirework(x, y) {
    const colors = ['#FFD700', '#FF5733', '#FF1493', '#00FFFF', '#32CD32', '#FFF'];
    for (let i = 0; i < 30; i++) {
        const particle = document.createElement('div');
        particle.className = 'firework-particle';
        particle.style.left = x + 'px';
        particle.style.top = y + 'px';
        particle.style.backgroundColor = colors[Math.floor(Math.random() * colors.length)];
        document.body.appendChild(particle);
        
        const angle = Math.random() * Math.PI * 2;
        const velocity = 60 + Math.random() * 120;
        const tx = Math.cos(angle) * velocity;
        const ty = Math.sin(angle) * velocity;
        
        requestAnimationFrame(() => {
            particle.style.transform = `translate(${tx}px, ${ty}px) scale(0)`;
            particle.style.opacity = '0';
        });
        setTimeout(() => { particle.remove(); }, 800);
    }
}

// 2. ==========================================
// INTERSECTION OBSERVER (Fade-in on scroll)
// ==========================================
export function applyScrollFadeEffects(rootElementSelector, itemSelector) {
    const rootEl = document.querySelector(rootElementSelector);
    if (!rootEl) return;

    const items = rootEl.querySelectorAll(itemSelector);
    if (items.length === 0) return;

    const observer = new IntersectionObserver((entries, obs) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
                obs.unobserve(entry.target); // Stop observing once visible
            }
        });
    }, { root: rootEl, threshold: 0.1 });

    // Stagger the observation slightly so they don't all pop at once
    setTimeout(() => { 
        items.forEach(n => observer.observe(n)); 
    }, 50);
}

// 3. ==========================================
// HOLOGRAPHIC 3D TILT (Mouse & Gyroscope)
// ==========================================
export function initHolographicTilt() {
    // Mouse Movement Logic
    document.addEventListener('mousemove', (e) => {
        // SAFETY FIX: Make sure e.target is a valid HTML element before running .closest()
        if (!e.target || typeof e.target.closest !== 'function') return;

        const target = e.target.closest('.holographic-active');
        if (!target) return;
        
        const rect = target.getBoundingClientRect();
        const x = e.clientX - rect.left, y = e.clientY - rect.top;
        const xPct = (x / rect.width) * 100, yPct = (y / rect.height) * 100;
        const tiltDeg = Math.floor((xPct + yPct) * 1.8) % 360; 
        
        target.style.setProperty('--mouse-x', (100 - xPct) + '%');
        target.style.setProperty('--mouse-y', (100 - yPct) + '%');
        target.style.setProperty('--tilt-deg', `${tiltDeg}deg`); 

        const pContainer = document.getElementById('modal-parallax');
        if (pContainer) {
            pContainer.style.setProperty('--move-x', `${(e.clientX - window.innerWidth / 2) / 8}px`);
            pContainer.style.setProperty('--move-y', `${(e.clientY - window.innerHeight / 2) / 8}px`);
        }
        
        const isMobile = window.innerWidth <= 768;
        const maxTilt = isMobile ? 3 : 8;
        const scaleAmt = isMobile ? 1 : 1.02;

        const rotateX = ((y / rect.height) - 0.5) * -maxTilt;
        const rotateY = ((x / rect.width) - 0.5) * maxTilt;
        
        if (target.classList.contains('plant-modal-content') && !isMobile) {
            target.style.transform = `perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale3d(${scaleAmt}, ${scaleAmt}, ${scaleAmt})`;
        } else if (target.classList.contains('plant-card')) {
            target.style.transform = `perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale3d(${scaleAmt}, ${scaleAmt}, ${scaleAmt})`;
            target.style.zIndex = "50";
        }
    });

    // Mouse Reset Logic
    document.addEventListener('mouseout', (e) => {
        if (!e.target || typeof e.target.closest !== 'function') return;
        const target = e.target.closest('.holographic-active');
        if (!target || target.contains(e.relatedTarget)) return;
        target.style.setProperty('--mouse-x', '50%');
        target.style.setProperty('--mouse-y', '50%');
        target.style.transform = `perspective(1000px) rotateX(0deg) rotateY(0deg) scale3d(1, 1, 1)`;
        target.style.zIndex = "1";
    });

    // Mobile Device Gyroscope Logic
    if (window.DeviceOrientationEvent) {
        window.addEventListener('deviceorientation', (e) => {
            let targets = document.querySelectorAll('.holographic-active');
            if (targets.length === 0) return; 
            
            let gamma = Math.max(-45, Math.min(45, e.gamma || 0));
            let beta = Math.max(0, Math.min(90, e.beta || 0));
            const xPct = ((gamma + 45) / 90) * 100, yPct = (beta / 90) * 100;
            const tiltDeg = Math.floor(Math.abs(gamma * 4) + Math.abs(beta * 3)) % 360; 

            targets.forEach(target => {
                // If a modal is open, we only want to tilt the modal, not the cards behind it
                if (document.getElementById("plant-modal").style.display === "flex" && !target.classList.contains('plant-modal-content')) {
                    return;
                }
                // No tilt on overview cards on mobile
                if (target.closest('.sv-grid-card')) return;

                if (target.classList.contains('plant-modal-content')) {
                    const pContainer = target.querySelector('#modal-parallax');
                    if (pContainer) {
                        pContainer.style.setProperty('--move-x', `${gamma * 1.5}px`);
                        pContainer.style.setProperty('--move-y', `${(beta - 45) * 1.5}px`);
                    }
                }
                target.style.setProperty('--mouse-x', (100 - xPct) + '%');
                target.style.setProperty('--mouse-y', (100 - yPct) + '%');
                target.style.setProperty('--tilt-deg', `${tiltDeg}deg`); 
                if (!target.classList.contains('plant-modal-content')) {
                    target.style.transform = `perspective(1000px) rotateX(${(beta - 45) / -4}deg) rotateY(${gamma / 4}deg)`;
                }
            });
        });
    }
}