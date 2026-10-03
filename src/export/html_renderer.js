/**
 * Hyperfine Benchmark Results - JavaScript Renderer
 *
 * This script processes benchmark data and generates interactive visualizations
 * using Plotly.js. It provides tabbed sections for different analysis views.
 */

// Initialize when the DOM is fully loaded
document.addEventListener("DOMContentLoaded", function () {
  // Process the data and compute derived metrics
  processData();

  // Set up the tabbed interface
  initTabs();

  // Render all visualizations
  renderSummaryTable();
  renderBoxplot();
  renderHistograms();
  renderProgressionPlot();
  renderAdvancedStats();
  detectAndRenderParameters();
});

/**
 * Process benchmark data and calculate relative speeds
 */
function processData() {
  // Helper to check if a value is a number
  const isNumber = (x) => typeof x === "number" && !isNaN(x);

  // Convert times to selected unit
  benchmarkData.forEach((result) => {
    // Apply unit conversion to main statistics
    if (isNumber(result.mean)) result.meanInUnit = result.mean * unitFactor;
    if (isNumber(result.min)) result.minInUnit = result.min * unitFactor;
    if (isNumber(result.max)) result.maxInUnit = result.max * unitFactor;
    if (isNumber(result.stddev))
      result.stddevInUnit = result.stddev * unitFactor;

    // Convert timing data
    if (result.times) {
      result.timesInUnit = result.times.map((t) => t * unitFactor);
    }
  });

  // Find the reference or fastest result for relative comparison
  let referenceResult;
  if (referenceCommand) {
    referenceResult = benchmarkData.find((r) => r.command === referenceCommand);
  }

  if (!referenceResult) {
    // If no reference was specified, use the fastest result
    const fastestMean = Math.min(...benchmarkData.map((r) => r.mean));
    referenceResult = benchmarkData.find((r) => r.mean === fastestMean);
  }

  // Mark reference and calculate relative speeds
  benchmarkData.forEach((result) => {
    result.is_reference = result.command === referenceResult.command;
    result.relative_speed = result.mean / referenceResult.mean;

    // Calculate relative stddev if both results have stddev
    if (result.stddev && referenceResult.stddev) {
      // Use propagation of uncertainty formula for division
      result.relative_stddev =
        result.relative_speed *
        Math.sqrt(
          Math.pow(result.stddev / result.mean, 2) +
            Math.pow(referenceResult.stddev / referenceResult.mean, 2)
        );
    }
  });
}

/**
 * Initialize tabbed interface
 */
function initTabs() {
  const tabButtons = document.querySelectorAll(".tab-button");

  tabButtons.forEach((button) => {
    button.addEventListener("click", () => {
      // Remove active class from all buttons and contents
      tabButtons.forEach((btn) => btn.classList.remove("active"));
      document.querySelectorAll(".tab-content").forEach((content) => {
        content.classList.remove("active");
      });

      // Add active class to clicked button and its content
      button.classList.add("active");
      const tabId = button.getAttribute("data-tab");
      document.getElementById(tabId + "-tab").classList.add("active");

      // Trigger resize event to make sure plots render correctly
      window.dispatchEvent(new Event("resize"));
    });
  });
}

/**
 * Format a number according to the selected unit with specified decimals
 */
function formatValue(value, decimals = 3) {
  if (value === undefined || value === null) return "";
  return value.toFixed(decimals);
}

/**
 * Create the summary table with benchmark results
 */
function renderSummaryTable() {
  let tableHtml = `
    <h2>Summary</h2>
    <table>
      <thead>
        <tr>
          <th>Command</th>
          <th>Mean [${unitShortName}]</th>
          <th>Min [${unitShortName}]</th>
          <th>Max [${unitShortName}]</th>
          <th>Relative</th>
        </tr>
      </thead>
      <tbody>
  `;

  benchmarkData.forEach((result) => {
    const rowClass = result.is_reference ? "reference" : "";
    const stddev = result.stddev
      ? ` <span class="stddev">± ${formatValue(result.stddevInUnit)}</span>`
      : "";
    const relStddev = result.relative_stddev
      ? ` <span class="stddev">± ${formatValue(
          result.relative_stddev,
          2
        )}</span>`
      : "";

    tableHtml += `
      <tr class="${rowClass}">
        <td>${escapeHtml(result.command)}</td>
        <td>${formatValue(result.meanInUnit)}${stddev}</td>
        <td>${formatValue(result.minInUnit)}</td>
        <td>${formatValue(result.maxInUnit)}</td>
        <td>${formatValue(result.relative_speed, 2)}x${relStddev}</td>
      </tr>
    `;
  });

  tableHtml += `
      </tbody>
    </table>
  `;

  document.getElementById("summary-table").innerHTML = tableHtml;
}

/**
 * Create the boxplot comparison
 */
function renderBoxplot() {
  const boxplotData = benchmarkData.map((result) => {
    return {
      y: result.timesInUnit || [],
      type: "box",
      name: result.command,
      boxpoints: "all",
      jitter: 0.3,
      pointpos: 0,
    };
  });

  const layout = {
    title: `Runtime Comparison (${unitName})`,
    yaxis: { title: `Time [${unitShortName}]` },
    margin: { l: 60, r: 30, t: 50, b: 50 },
    autosize: true,
    responsive: true,
  };

  const config = {
    responsive: true,
    displayModeBar: false,
  };

  Plotly.newPlot("boxplot", boxplotData, layout, config);
}

/**
 * Create histograms for each command
 */
function renderHistograms() {
  const histogramsContainer = document.getElementById("histograms");
  histogramsContainer.innerHTML = "";

  benchmarkData.forEach((result, index) => {
    // Create a div for this histogram
    const chartDiv = document.createElement("div");
    chartDiv.className = "chart-box";
    chartDiv.innerHTML = `<h3>${escapeHtml(
      result.command
    )}</h3><div id="histogram-${index}" class="chart"></div>`;
    histogramsContainer.appendChild(chartDiv);

    // Only create histogram if we have timing data
    if (result.timesInUnit && result.timesInUnit.length > 0) {
      const histData = [
        {
          x: result.timesInUnit,
          type: "histogram",
          marker: { color: "rgba(100, 200, 102, 0.7)" },
        },
      ];

      const layout = {
        title: "Distribution of runtimes",
        xaxis: { title: `Time [${unitShortName}]` },
        yaxis: { title: "Count" },
        margin: { l: 50, r: 30, t: 50, b: 50 },
        autosize: true,
        responsive: true,
      };

      const config = {
        responsive: true,
        displayModeBar: false,
      };

      Plotly.newPlot(`histogram-${index}`, histData, layout, config);
    } else {
      document.getElementById(`histogram-${index}`).innerHTML =
        "<p>No detailed timing data available for this command.</p>";
    }
  });
}

/**
 * Create time progression plot with moving averages
 */
function renderProgressionPlot() {
  const plotDiv = document.getElementById("progression");

  // Define color palette for consistent colors
  const defaultColors = [
    "#1f77b4",
    "#ff7f0e",
    "#2ca02c",
    "#d62728",
    "#9467bd",
    "#8c564b",
    "#e377c2",
    "#7f7f7f",
    "#bcbd22",
    "#17becf",
  ];

  const traces = [];
  const movingAverageTraces = [];
  const colorMap = {};

  benchmarkData.forEach((result, idx) => {
    if (result.timesInUnit && result.timesInUnit.length > 0) {
      // Create array of iteration indices
      const iterations = Array.from(
        { length: result.timesInUnit.length },
        (_, i) => i + 1
      );

      // Assign a color to this command
      colorMap[result.command] = defaultColors[idx % defaultColors.length];

      // Create scatter trace for individual points
      traces.push({
        x: iterations,
        y: result.timesInUnit,
        mode: "markers",
        name: result.command,
        marker: {
          color: colorMap[result.command],
          size: 5,
          opacity: 0.7,
        },
      });

      // Calculate moving average with adaptive window size
      const windowSize = Math.max(3, Math.floor(result.timesInUnit.length / 5));
      const movingAvg = calculateMovingAverage(result.timesInUnit, windowSize);

      // Create line trace for moving average with same color
      movingAverageTraces.push({
        x: iterations,
        y: movingAvg,
        mode: "lines",
        name: `${result.command} (moving avg)`,
        line: {
          color: colorMap[result.command],
          width: 2,
          dash: "solid",
        },
        showlegend: false, // Don't show in legend to avoid cluttering
      });
    }
  });

  // Combine all traces
  const allTraces = [...traces, ...movingAverageTraces];

  const layout = {
    title: "Time Progression",
    xaxis: { title: "Iteration number" },
    yaxis: { title: `Time [${unitShortName}]` },
    hovermode: "closest",
    autosize: true,
    responsive: true,
  };

  const config = {
    responsive: true,
    displayModeBar: false,
  };

  Plotly.newPlot(plotDiv, allTraces, layout, config);
}

/**
 * Calculate moving average of a time series
 */
function calculateMovingAverage(values, windowSize) {
  const result = [];

  for (let i = 0; i < values.length; i++) {
    let windowStart = Math.max(0, i - Math.floor(windowSize / 2));
    let windowEnd = Math.min(values.length, i + Math.ceil(windowSize / 2));
    let sum = 0;

    for (let j = windowStart; j < windowEnd; j++) {
      sum += values[j];
    }

    result.push(sum / (windowEnd - windowStart));
  }

  return result;
}

/**
 * Create advanced statistics cards
 */
function renderAdvancedStats() {
  const container = document.getElementById("stats-container");

  benchmarkData.forEach((result) => {
    if (!result.timesInUnit || result.timesInUnit.length === 0) return;

    const times = result.timesInUnit;
    const sorted = [...times].sort((a, b) => a - b);

    // Calculate key statistics
    const mean = times.reduce((a, b) => a + b) / times.length;
    const min = sorted[0];
    const max = sorted[sorted.length - 1];
    const median = quantile(sorted, 0.5);
    const p5 = quantile(sorted, 0.05);
    const p25 = quantile(sorted, 0.25);
    const p75 = quantile(sorted, 0.75);
    const p95 = quantile(sorted, 0.95);
    const iqr = p75 - p25;

    // Calculate variance and standard deviation
    const variance =
      times.reduce((acc, val) => acc + Math.pow(val - mean, 2), 0) /
      (times.length - 1);
    const stddev = Math.sqrt(variance);

    // Create stats card with formatted values
    const card = document.createElement("div");
    card.className = "stats-card";

    card.innerHTML = `
      <div class="stats-header">${escapeHtml(result.command)}</div>
      <div class="stats-row">
        <span class="stats-label">Runs:</span>
        <span class="stats-value">${times.length}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">Mean:</span>
        <span class="stats-value">${formatValue(mean)} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">Std dev:</span>
        <span class="stats-value">${formatValue(stddev)} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">Median:</span>
        <span class="stats-value">${formatValue(median)} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">Min:</span>
        <span class="stats-value">${formatValue(min)} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">Max:</span>
        <span class="stats-value">${formatValue(max)} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">P_05..P_95:</span>
        <span class="stats-value">${formatValue(p5)}..${formatValue(
      p95
    )} ${unitShortName}</span>
      </div>
      <div class="stats-row">
        <span class="stats-label">P_25..P_75 (IQR):</span>
        <span class="stats-value">${formatValue(p25)}..${formatValue(
      p75
    )} ${unitShortName} (${formatValue(iqr)} ${unitShortName})</span>
      </div>
    `;

    container.appendChild(card);
  });
}

/**
 * Compute the q-th quantile from sorted array values
 * Uses the R7 method, which is default in NumPy/SciPy
 *
 * @param {Array} sorted - Sorted array of values
 * @param {number} q - Quantile to compute (0 <= q <= 1)
 * @return {number} The q-th quantile value
 */
function quantile(sorted, q) {
  if (q <= 0) return sorted[0];
  if (q >= 1) return sorted[sorted.length - 1];

  const n = sorted.length;
  const index = (n - 1) * q;
  const low = Math.floor(index);
  const high = Math.ceil(index);
  const h = index - low;

  return (1 - h) * sorted[low] + h * sorted[high];
}

/**
 * Detect parameters in commands and render parameter plot
 */
function detectAndRenderParameters() {
  // Extract unique parameter names
  const parameterNames = new Set();
  benchmarkData.forEach((result) => {
    if (result.parameters) {
      Object.keys(result.parameters).forEach((param) =>
        parameterNames.add(param)
      );
    }
  });

  if (parameterNames.size === 0) return;

  // Show the parameter tab button
  document.getElementById("param-tab-button").style.display = "block";

  // Create plot for each parameter
  const plotDiv = document.getElementById("param-plot");
  const traces = [];

  parameterNames.forEach((paramName) => {
    // Get results for this parameter
    const relevantResults = benchmarkData.filter(
      (result) => result.parameters && result.parameters[paramName]
    );

    if (relevantResults.length < 2) return; // Need at least 2 points for a line

    // Create data points
    const dataPoints = relevantResults.map((result) => ({
      value: parseFloat(result.parameters[paramName]),
      mean: result.mean,
      stddev: result.stddev ? result.stddev : 0,
      command: result.command,
    }));

    // Sort by parameter value
    dataPoints.sort((a, b) => a.value - b.value);

    // Create trace for this parameter
    traces.push({
      x: dataPoints.map((p) => p.value),
      y: dataPoints.map((p) => p.mean),
      error_y: {
        type: "data",
        array: dataPoints.map((p) => p.stddev),
        visible: true,
      },
      mode: "lines+markers",
      type: "scatter",
      name: paramName,
      hovertemplate: `${paramName}=%{x}: %{y:.3f} ${unitShortName} ± %{error_y.array:.3f}<br>%{text}`,
      text: dataPoints.map((p) => p.command),
    });
  });

  const layout = {
    title: "Parameter Analysis",
    xaxis: { title: "Parameter Value" },
    yaxis: {
      title: `Time [${unitShortName}]`,
      rangemode: "tozero",
    },
    hovermode: "closest",
    autosize: true,
    responsive: true,
  };

  const config = {
    responsive: true,
    displayModeBar: false,
  };

  Plotly.newPlot(plotDiv, traces, layout, config);
}

/**
 * Escape HTML special characters for safe display
 */
function escapeHtml(unsafe) {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/<\//g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
