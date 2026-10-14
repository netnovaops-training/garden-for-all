// =============================================================================
// js/weather.js — Garden weather forecast widget
//
// Reads the zip code from the data-zip attribute on #forecast-container,
// injected by sections/weather.php. No PHP inside this file.
// =============================================================================
(function () {
    'use strict';

    async function loadReliableForecast() {
        const container = document.getElementById('forecast-container');
        const toggleBtn = document.getElementById('toggle-forecast-btn');
        if (!container) return;

        const zip = (container.dataset.zip || '').trim();
        if (!zip) return;

        try {
            // Step 1: Zip → coordinates
            const locRes = await fetch('https://api.zippopotam.us/us/' + zip);
            if (!locRes.ok) throw new Error('Zip code not found.');
            const locData = await locRes.json();
            const lat = locData.places[0].latitude;
            const lon = locData.places[0].longitude;

            // Step 2: Weather + Air Quality (parallel)
            const wUrl  = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&hourly=temperature_2m,relative_humidity_2m,precipitation_probability,wind_speed_10m,uv_index,is_day&temperature_unit=fahrenheit&wind_speed_unit=mph&forecast_days=2&timezone=auto`;
            const aqUrl = `https://air-quality-api.open-meteo.com/v1/air-quality?latitude=${lat}&longitude=${lon}&hourly=us_aqi&forecast_days=2&timezone=auto`;

            const [weatherRes, aqRes] = await Promise.all([fetch(wUrl), fetch(aqUrl)]);
            const wData  = await weatherRes.json();
            const aqData = await aqRes.json();

            if (!wData.hourly)             throw new Error('Weather data currently unavailable.');
            if (!aqData.hourly?.us_aqi)    throw new Error('Air quality data currently unavailable.');

            let html = '';
            let daytimeCount = 0;

            for (let i = 0; i < wData.hourly.time.length; i++) {
                if (wData.hourly.is_day[i] === 0) continue;

                const temp      = Math.round(wData.hourly.temperature_2m[i]);
                const hum       = Math.round(wData.hourly.relative_humidity_2m[i]);
                const rainProb  = wData.hourly.precipitation_probability[i];
                const windSpeed = Math.round(wData.hourly.wind_speed_10m[i]);
                const uv        = Math.round(wData.hourly.uv_index[i]);
                const aqi       = aqData.hourly.us_aqi[i] !== null ? Math.round(aqData.hourly.us_aqi[i]) : 0;

                const dateObj  = new Date(wData.hourly.time[i]);
                const now      = new Date();
                const dayLabel = dateObj.getDate() === now.getDate() ? 'TODAY' : 'TOMORROW';
                const timeLabel = dateObj.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });

                // Scoring
                let score = 100;
                if      (temp < 32)  score -= 70;
                else if (temp < 45)  score -= 45;
                else if (temp < 52)  score -= 30;
                else if (temp > 90)  score -= 65;
                else if (temp > 85)  score -= 25;
                if      (rainProb > 75) score -= 65;
                else if (rainProb > 40) score -= 35;
                else if (rainProb > 15) score -= 15;
                if      (windSpeed >= 20) score -= 65;
                else if (windSpeed >= 12) score -= 35;
                else if (windSpeed >   7) score -= 10;
                if  (aqi > 100) score -= 65;
                else if (aqi > 50) score -= 15;
                if (hum > 85) score -= 15;
                if (uv  > 7)  score -= 20;
                score = Math.max(0, Math.min(100, score));

                let themeBg = '#4CAF50';
                let v       = 'PERFECT 🌱';

                if (score < 40) {
                    themeBg = '#E57373';
                    if      (aqi > 100)       v = 'POOR AIR QUALITY 😷';
                    else if (temp < 32)        v = 'SNOW & COLD ❄️';
                    else if (windSpeed >= 20)  v = 'CROP DANGER! 🌪️';
                    else if (rainProb > 75)    v = 'POURING RAIN 🌊';
                    else if (temp > 90)        v = 'EXTREME HEAT 🥵';
                    else                       v = 'POOR CONDITIONS 🛑';
                } else if (score < 80) {
                    themeBg = '#FFA726';
                    if      (temp < 52)               v = 'CHILLY 🧥';
                    else if (windSpeed >= 12)          v = 'TOO WINDY 💨';
                    else if (rainProb > 40)            v = 'EXPECT RAIN 🌧️';
                    else if (temp > 85 && hum > 70)    v = 'MUGGY & HOT 🥵';
                    else if (uv > 7)                   v = 'HIGH UV! ☀️';
                    else                               v = 'FAIR CONDITIONS ⛅';
                }

                const displayStyle = daytimeCount >= 6 ? 'display:none;' : 'display:flex;';
                const extraClass   = daytimeCount >= 6 ? 'extra-hour-card' : '';
                daytimeCount++;

                html += `
                <div class="${extraClass}" style="background:rgba(255,255,255,0.2);border:4px solid #333;border-radius:15px;padding:20px;box-shadow:4px 4px 0px rgba(0,0,0,0.2);text-align:left;${displayStyle}flex-direction:column;justify-content:space-between;">
                    <div>
                        <div class="blog-cartoony-text" style="font-size:0.9rem;margin-bottom:5px;opacity:0.8;font-weight:bold;color:#333;">${dayLabel} @ ${timeLabel}</div>
                        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;">
                            <span class="blog-cartoony-text" style="font-size:2.2rem;line-height:1;color:#333;">${temp}°F</span>
                            <span class="blog-cartoony-text" style="font-size:1.1rem;background:${themeBg};padding:5px 10px;border-radius:10px;border:2px solid #333;color:#fff!important;font-weight:900;text-shadow:1px 1px 0px rgba(0,0,0,0.2);">Score: ${score}</span>
                        </div>
                        <div class="blog-cartoony-text" style="font-size:0.85rem;line-height:1.6;margin-bottom:15px;color:#333;">
                            💧 Hum: ${hum}% | 🌧️ Rain: ${rainProb}%<br>🍃 Wind: ${windSpeed} mph<br>☀️ UV: ${uv} | 💨 AQI: ${aqi}
                        </div>
                    </div>
                    <div class="blog-cartoony-text" style="background:${themeBg};border:3px solid #333;padding:8px;border-radius:10px;font-size:0.85rem;text-align:center;font-weight:bold;color:#fff!important;text-shadow:1px 1px 0px rgba(0,0,0,0.2);">${v}</div>
                </div>`;
            }

            if (!html) {
                container.innerHTML = '<div class="blog-cartoony-text" style="grid-column:1/-1;">No daytime hours remaining for the current period. Check back tomorrow!</div>';
                if (toggleBtn) toggleBtn.style.display = 'none';
                return;
            }

            container.innerHTML = html;

            if (daytimeCount > 6 && toggleBtn) {
                toggleBtn.style.display = 'inline-block';
                toggleBtn.onclick = function () {
                    const cards      = document.querySelectorAll('.extra-hour-card');
                    const isExpanded = toggleBtn.getAttribute('data-expanded') === 'true';
                    cards.forEach(c => { c.style.display = isExpanded ? 'none' : 'flex'; });
                    toggleBtn.innerHTML          = isExpanded ? 'SHOW MORE 🔽' : 'SHOW LESS 🔼';
                    toggleBtn.setAttribute('data-expanded', String(!isExpanded));
                };
            } else if (toggleBtn) {
                toggleBtn.style.display = 'none';
            }

        } catch (err) {
            container.innerHTML = `<div class="blog-cartoony-text" style="grid-column:1/-1;color:#ff5252!important;">⚠️ Error: ${err.message}</div>`;
            if (toggleBtn) toggleBtn.style.display = 'none';
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', loadReliableForecast);
    } else {
        loadReliableForecast();
    }
}());
