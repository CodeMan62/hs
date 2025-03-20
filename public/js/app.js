// app.js - Simple JavaScript for testing static file serving
document.addEventListener('DOMContentLoaded', function() {
    console.log('Static JS file loaded successfully!');
    
    // Add a click counter to demonstrate JS functionality
    let counter = 0;
    
    // Find any button elements on the page
    const buttons = document.querySelectorAll('.btn');
    buttons.forEach(button => {
        button.addEventListener('click', function() {
            counter++;
            document.getElementById('counter-value').textContent = counter;
            
            // Change background color based on click count
            const colors = ['#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6'];
            this.style.backgroundColor = colors[counter % colors.length];
        });
    });
    
    // Add the counter display to the page
    const counterDisplay = document.createElement('div');
    counterDisplay.innerHTML = `
        <div class="container" style="margin-top: 20px;">
            <h3>Click Counter</h3>
            <p>You've clicked <span id="counter-value">0</span> times.</p>
        </div>
    `;
    document.body.appendChild(counterDisplay);
    
    // Add current time to the page
    const timeDisplay = document.createElement('div');
    timeDisplay.classList.add('container');
    timeDisplay.style.marginTop = '20px';
    timeDisplay.innerHTML = `
        <h3>Server Time</h3>
        <p>This page was loaded at: ${new Date().toLocaleTimeString()}</p>
    `;
    document.body.appendChild(timeDisplay);
    
    // Add a simple animation to demonstrate that JS is working
    const highlight = document.createElement('div');
    highlight.classList.add('container');
    highlight.style.marginTop = '20px';
    highlight.innerHTML = `
        <h3>Animation Demo</h3>
        <div id="animated-box" style="width: 100px; height: 100px; background-color: #3498db; transition: all 0.5s ease;"></div>
    `;
    document.body.appendChild(highlight);
    
    // Animate the box
    const animatedBox = highlight.querySelector('#animated-box');
    let growing = true;
    
    setInterval(() => {
        if (growing) {
            animatedBox.style.width = '200px';
            animatedBox.style.backgroundColor = '#e74c3c';
            growing = false;
        } else {
            animatedBox.style.width = '100px';
            animatedBox.style.backgroundColor = '#3498db';
            growing = true;
        }
    }, 1500);
});