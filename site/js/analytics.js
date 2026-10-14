// js/analytics.js
document.addEventListener('DOMContentLoaded', () => {
    const data = window.CHART_DATA;
    if (!data || !data.monthly) return;

    const ctx = document.getElementById('monthlyActivityChart');
    if (!ctx || !data.monthly.labels.length) return;

    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: data.monthly.labels,
            datasets: [{
                data: data.monthly.data,
                backgroundColor: '#e65100',
                borderColor: '#fff',
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            plugins: {
                title: { display: true, text: 'Active Tasks per Month', font: { size: 16 } },
                legend: { display: false }
            },
            elements: { point: { radius: 3 } }
        }
    });
});