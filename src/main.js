const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    console.log("submit", e);
    console.log("greetInputEl", greetInputEl.value);
    greet();
  });
});


// 获取按钮和输入框的元素
const analyzeButton = document.getElementById("analyze-btn");
const directoryPathInput = document.getElementById("directory-path");
const filesList = document.getElementById("files-list");
const foldersList = document.getElementById("folders-list");
const loadingIndicator = document.getElementById("loading-indicator");

// 处理分析的函数（相同逻辑）
async function analyzeDirectory() {
  // 获取用户输入的目录路径
  const path = directoryPathInput.value.trim();

  if (!path) {
    alert("Please enter a valid directory path.");
    return;
  }

  // 设置文件大小限制，这里示例为10MB（单位：字节）
  const sizeLimit = 10 * 1024;  // 10KB 示例，修改为需要的大小

  // 显示加载指示器
  loadingIndicator.style.display = "block";

  try {
    console.log("Requesting analysis for path:", path, "with size limit:", sizeLimit);

    // 调用 Tauri 后端命令 `analyze_directory` 并传递路径和大小限制
    const result = await invoke("analyze_directory", {
      path: "/Users/leimao/Documents/nfclean",
      sizeLimit: sizeLimit,
    });

    // 将分析结果存储到 localStorage
    localStorage.setItem("analysisResult", JSON.stringify(result));

    // 输出返回的结果（文件列表）
    console.log("Directory analysis result:", result);

    // 渲染文件列表
    renderFileList(result);

  } catch (error) {
    console.error("Error analyzing directory:", error);
    alert("Error analyzing directory: " + error);
  } finally {
    // 隐藏加载指示器
    loadingIndicator.style.display = "none";
  }
}

// 点击分析按钮时触发分析
analyzeButton.addEventListener("click", analyzeDirectory);

// 监听输入框的回车键事件
directoryPathInput.addEventListener("keydown", function (event) {
  if (event.key === "Enter") {
    // 阻止默认行为（防止表单提交）
    event.preventDefault();

    // 调用相同的分析逻辑
    analyzeDirectory();
  }
});



// 渲染文件列表
function renderFileList(fileList) {
  const fileListContainer = document.getElementById('file-list');
  fileListContainer.innerHTML = ''; // 清空当前内容

  // 如果文件列表为空，显示信息
  if (fileList.length === 0) {
    fileListContainer.innerHTML = '<p class="text-gray-500">No files found matching the criteria.</p>';
    return;
  }

  // 遍历返回的文件列表，生成 HTML 元素
  fileList.forEach((item) => {
    const fileElement = document.createElement('div');
    fileElement.classList.add('flex', 'items-center', 'space-x-4', 'py-2', 'border-b', 'border-gray-200');

    // 创建一个文件/文件夹项的描述
    const fileDesc = document.createElement('p');
    fileDesc.textContent = `${item.path} - ${item.size_str}`;
    fileDesc.classList.add('text-gray-800', 'text-sm');  // 使用 Tailwind 类来控制文本颜色和大小

    // 勾选框
    const checkBox = document.createElement('input');
    checkBox.type = 'checkbox';
    checkBox.value = item.file_path; // 存储文件路径，以便删除时使用
    checkBox.classList.add('h-5', 'w-5', 'text-blue-600', 'focus:ring-0');  // 勾选框样式

    // 创建文件夹图标
    const icon = document.createElement('span');
    icon.classList.add('text-xl', 'mr-3');  // 使用更大图标并调整间距
    if (!item.is_file) {
      icon.textContent = '📁'; // 文件夹图标
    } else {
      icon.textContent = '📄'; // 文件图标
    }

    // 将图标、勾选框和描述添加到文件项中
    fileElement.appendChild(icon);
    fileElement.appendChild(checkBox);
    fileElement.appendChild(fileDesc);

    fileListContainer.appendChild(fileElement);
  });
}

