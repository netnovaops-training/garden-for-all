// js/home.js

const config = window.HOME_CONFIG || {};

// 1. SLIDESHOW 
const homeSlides = document.querySelectorAll('.hero-slide');
let currentSlide = 0;
function nextSlide() {
    if (homeSlides.length > 0) {
        homeSlides[currentSlide].classList.remove('active');
        currentSlide = (currentSlide + 1) % homeSlides.length;
        homeSlides[currentSlide].classList.add('active');
    }
}
setInterval(nextSlide, 8000); 

// 2. FACTS
const factText = document.getElementById("fun-fact-text");
const homeOverlayBox = document.getElementById('overlay-box'); // Renamed to prevent crashes

function updateFact() {
    if (config.allFacts && config.allFacts.length > 0) {
        homeOverlayBox.style.opacity = 0; 
        setTimeout(() => {
            const randomFactIndex = Math.floor(Math.random() * config.allFacts.length);
            factText.innerText = config.allFacts[randomFactIndex];
            homeOverlayBox.style.opacity = 1; 
        }, 900); 
    }
}
updateFact();
setInterval(updateFact, 9000); 

// 3. WEATHER & RADAR
const radarFrame = document.getElementById('radarFrame');

async function getWeather() {
    const zip = document.getElementById('zipCode').value;
    if (zip.length !== 5 || isNaN(zip)) {
        alert("Please enter a valid 5-digit US Zip Code.");
        return;
    }

    localStorage.setItem('gardenZip', zip);
    const grid = document.getElementById('weather-grid');
    grid.innerHTML = '<div style="grid-column: 1/-1; text-align:center;">Checking soil & sky...</div>';

    try {
        const locRes = await fetch(`https://api.zippopotam.us/us/${zip}`);
        if (!locRes.ok) throw new Error("Zip code not found");
        const locData = await locRes.json();
        const lat = locData.places[0].latitude;
        const lon = locData.places[0].longitude;

        if (radarFrame) {
            radarFrame.src = `https://embed.windy.com/embed2.html?lat=${lat}&lon=${lon}&detailLat=${lat}&detailLon=${lon}&width=650&height=450&zoom=10&level=surface&overlay=rain&product=ecmwf&menu=&message=&marker=&calendar=now&pressure=&type=map&location=coordinates&detail=&metricWind=default&metricTemp=default&radarRange=-1`;
        }

        const weatherRes = await fetch(`https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&daily=weathercode,temperature_2m_max,precipitation_probability_max&temperature_unit=fahrenheit&forecast_days=3&timezone=auto`);
        const wData = await weatherRes.json();

        let html = ``;
        const days = ["Today", "Tomorrow", "Day 3"];
        
        for(let i=0; i<3; i++) {
            const maxTemp = wData.daily.temperature_2m_max[i];
            const rainProb = wData.daily.precipitation_probability_max[i];

            let verdict = "Decent Day to Plant! 🌤️";
            let vClass = "v-ok";

            if (rainProb > 60) { verdict = "Rainy Day 🌧️"; vClass = "v-bad"; }
            else if (maxTemp < 45) { verdict = "Too Cold to Garden ❄️"; vClass = "v-bad"; }
            else if (maxTemp > 90) { verdict = "Too Hot to Garden 🔥"; vClass = "v-bad"; }
            else if (maxTemp >= 60 && maxTemp <= 85 && rainProb < 20) { verdict = "Perfect Day to Garden! 🌿"; vClass = "v-good"; }

            html += `
                <div class="weather-card">
                    <strong>${days[i]}</strong><br>
                    <span style="font-size:1.5rem;">${maxTemp}°F</span><br>
                    <small>Rain: ${rainProb}%</small><br>
                    <span class="verdict-badge ${vClass}">${verdict}</span>
                </div>
            `;
        }
        grid.innerHTML = html;

    } catch (err) {
        console.error(err);
        grid.innerHTML = '<div style="color:red; text-align:center;">Could not fetch weather. Check Zip Code.</div>';
    }
}

// Auto-load weather if zip exists
const savedZip = localStorage.getItem('gardenZip');
if (savedZip) {
    document.getElementById('zipCode').value = savedZip;
    getWeather();
}

// Expose to window for the HTML button
window.getWeather = getWeather;