// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

window.onload = function () {
  document.getElementById("url").value = "";
};

async function pollResults(task_id) {
  const response = await fetch(`/crawler-result/${task_id}`);
  const data = await response.json();
  if (data.ready) {
    console.log("Results:", data.results);
  } else {
    setTimeout(() => pollResults(task_id), 2000);
  }
}

async function run_crawler() {
  const resultsDiv = document.getElementById("results");

  if (document.getElementById("url").value === "") {
    return;
  } else if (
    !document.getElementById("url").value.includes("http") ||
    !document.getElementById("url").value.includes("https")
  ) {
    const div = document.createElement("div");
    div.className = "error";
    div.innerHTML = `<br>
        <span style="color: #d3ac66;" class="result_text">Insira uma URL válida que inclua o protocolo de transferência (https:// ou http://)</span>`;
    resultsDiv.appendChild(div);
    return;
  }

  const subtitle = document.getElementById("subtitle");
  const url_input = document.getElementById("url");
  const buttons = document.getElementsByClassName("buttons")[0];
  const spinner = document.getElementById("spinner");

  subtitle.style.display = "none";
  url_input.style.display = "none";
  buttons.style.display = "none";

  resultsDiv.innerHTML = "";
  spinner.style.display = "block";

  const startResp = await fetch("/run-crawler", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ url: url_input.value }),
  });
  const startData = await startResp.json();
  if (!startData.success) return;

  const task_id = startData.task_id;

  const pollInterval = 2000;
  const pollResults = async () => {
    const res = await fetch(`/crawler-result/${task_id}`);
    const data = await res.json();

    if (!data.ready) {
      setTimeout(pollResults, pollInterval);
      return;
    }

    spinner.style.display = "none";

    resultsDiv.innerHTML = `
        <h2 style="margin-bottom: 28px; color: #d3ac66; font-weight: 500">
            ${url_input.value}
        </h2>
        `;

    data.results.forEach((r) => {
      const icon = r.passed ? "✔️" : "❌";
      const div = document.createElement("div");
      div.className = "result_item";
      div.innerHTML = `<span class="result_icon">${icon}</span> 
                    <span style="color: #d3ac66;" class="result_text">${r.check}</span>`;
      resultsDiv.appendChild(div);
    });

    const div_refresh = document.createElement("div");
    div_refresh.className = "refresh_item";
    div_refresh.innerHTML =
      '<button id="refresh_button" onclick="window.location.reload();">Nova consulta</button>';
    resultsDiv.appendChild(div_refresh);
  };

  pollResults();
}
