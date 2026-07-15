const invoiceStatus = document.querySelector("#invoice-status");
const invoiceCount = document.querySelector("#invoice-count");
const invoiceFiles = document.querySelector("#invoice-files");
const refreshInvoiceCount = () => { invoiceCount.textContent = String(invoiceFiles.children.length); };

document.querySelectorAll(".invoice-nav button").forEach((button) => {
  button.addEventListener("click", () => {
    document.querySelector(".invoice-nav button.selected")?.classList.remove("selected");
    button.classList.add("selected");
  });
});

invoiceFiles.addEventListener("click", (event) => {
  if (!event.target.classList.contains("invoice-remove")) return;
  event.target.closest(".invoice-file-card").remove();
  refreshInvoiceCount();
  invoiceStatus.textContent = "已移除一张发票";
});

document.querySelector("#invoice-add").addEventListener("click", () => {
  const card = invoiceFiles.firstElementChild?.cloneNode(true);
  if (!card) return;
  card.querySelector("h2").textContent = `新增发票_${invoiceFiles.children.length + 1}.pdf`;
  card.querySelector("p").textContent = "原文件：新添加文件.pdf · 等待确认";
  invoiceFiles.append(card);
  refreshInvoiceCount();
  invoiceStatus.textContent = "已添加一张待处理发票";
});

document.querySelector("#invoice-rename").addEventListener("click", () => {
  invoiceStatus.textContent = `已完成 ${invoiceFiles.children.length} 张发票重命名`;
});
