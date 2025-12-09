<template>
  <main class="container">
    <h1>分组器</h1>
    <p class="save-info">* 人员名单会自动保存，下次打开应用时仍会保留</p>

    <!-- 配置区域 -->
    <div class="config-section">
      <div class="config-item">
        <label for="total-people">待分组人数：</label>
        <div>{{ waitingForGroup.length }}</div>
        <!-- <select
          id="total-people"
          v-model.number="totalPeople"
          @change="updateTotalPeople(totalPeople)"
        >
          <option value="4">4人</option>
          <option value="6">6人</option>
          <option value="8">8人</option>
        </select> -->
      </div>

      <div class="config-item">
        <label for="group-count">分组数：</label>
        <select
          id="group-count"
          v-model.number="groupCount"
        >
          <option
            v-for="n in validGroupOptions"
            :key="n"
            :value="n"
          >{{ n }}组</option>
        </select>
      </div>

      <!-- 编辑模式切换 -->
      <div class="config-item">
        <!-- v-show="!isRandomized" -->
        <label>显示模式：</label>
        <button
          @click="toggleEditMode"
          class="mode-btn"
        >
          {{ editMode ? '编辑模式' : '分组模式' }}
        </button>
      </div>
    </div>

    <!-- 人员列表 -->
    <div>

      <!-- 人员编辑 -->
      <div
        class="horizontal-layout"
        v-if="editMode"
      >
        <div class="edit-panel">
          <!-- 人员名单编辑区域 -->
          <div class="names-list-section">
            <div class="names-list-title ">点击人名快捷删除</div>
            <div class="names-list custom-scroll">
              <div
                v-for="(name, index) in CacheGroups"
                :key="index"
                class="edit-list-item name-text"
                @click="removeFromCacheGroups(index)"
              >
                <span class="name-text">{{ name }}</span>
                <span class="arrow-icon del-icon">×</span>
              </div>
            </div>
          </div>
        </div>
        <!-- 快捷添加区域 -->
        <div>
          <div>输入成员名字，以空格或回车分隔</div>
          <textarea
            name=""
            cols="30"
            rows="10"
            v-model="textareaInput"
            id="addRoles"
          ></textarea>
        </div>
      </div>

      <!-- 待分组 -->
      <div
        class="horizontal-layout"
        v-else
      >
        <div class="left-panel">
          <!-- 人员名单编辑区域 -->
          <div class="names-list-section">
            <div class="names-list-title">待分组 ({{ waitingForGroup.length }}人)</div>
            <div class="names-list custom-scroll">
              <div
                v-for="(name, index) in waitingForGroup"
                :key="index"
                class="name-list-item name-text  "
                @click="removeFromWaitingGroup(index, name)"
              >
                <div class="impatient">
                  {{ name }}
                </div>
                <!-- <span class="name-text">{{ name }}</span> -->
                <!-- <span class="arrow-icon remove-icon">→</span> -->
              </div>
            </div>
          </div>
        </div>
        <div class="changeArrow">⇋</div>
        <!-- 右侧：人员名单区域 -->
        <div class="left-panel">
          <!-- 人员名单编辑区域 -->
          <div class="names-list-section">
            <div class="names-list-title ">点击人名快捷添加</div>
            <div class="names-list custom-scroll">
              <div
                v-for="(name, index) in remainingGroup"
                :key="index"
                class="name-list-item name-text"
                @click="addToWaitingGroup(index, name)"
              >
                <!-- <span class="arrow-icon add-icon">←</span> -->
                <!-- <span class="name-text">{{ name }}</span> -->
                {{ name }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div>
      <div
        class="actions"
        v-if="!editMode"
      >
        <button
          @click="randomizeGroups"
          :disabled="isRandomized"
          class="primary-btn"
        >
          {{ isRandomized ? '已分组' : '开始分组' }}
        </button>
        <button
          @click="regroup"
          v-if="isRandomized"
          class="secondary-btn"
        >
          重新分组
        </button>
        <button
          @click="showResultInPopup"
          v-if="isRandomized && groups.length > 0"
          class="secondary-btn"
        >
          查看分组结果
        </button>
      </div>
      <div
        class="actions"
        v-else
      >
        <button
          @click="handleAddRolesInput"
          class="secondary-btn"
        >
          添加→分组模式
        </button>
      </div>
    </div>
  </main>
</template>
<script setup lang="ts">
  import { ref, computed, onMounted } from "vue";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from '@tauri-apps/api/event';
  import { WebviewWindow, getAllWebviewWindows } from '@tauri-apps/api/webviewWindow';
  // 定义分组类型
  interface Group {
    id: number;
    members: string[];
    points: number;
  }

  // 响应式数据

  const groupCount = ref(4); // 初始值设为4，但需要根据实际情况调整
  const peopleNames = ref<string[]>([]);
  const waitingForGroup = ref<string[]>([]);
  const CacheGroups = ref<string[]>([]);
  const groups = ref<Group[]>([]);
  const isRandomized = ref(false);
  const editMode = ref(false); // 编辑模式开关
  const textareaInput = ref('');

  // 实时计算
  // 根据总人数和组数计算每组人数
  const peoplePerGroup = computed(() => {
    return Math.floor(totalPeople.value / groupCount.value);
  });
  const totalPeople = computed(() => waitingForGroup.value.length);
  // 剩余组
  const remainingGroup = computed(() => CacheGroups.value.filter(i => !waitingForGroup.value.includes(i)));
  // 剩余人数（需要分配到某些组中）
  const remainingPeople = computed(() => {
    return totalPeople.value % groupCount.value;
  });

  // 新增：合理的分组数选项
  const validGroupOptions = computed(() => {
    const total = totalPeople.value;
    if (total < 2) return []; // 至少需要2人才能分组

    const options = [];
    // 从2组开始，最多到总人数的一半（确保每组至少有2人）
    for (let n = 2; n <= Math.floor(total / 2); n++) {
      // 检查分组是否合理：每组人数至少为2，且剩余人数不超过组数
      const groupSize = Math.floor(total / n);

      if (groupSize >= 2) {
        options.push(n);
      }
    }
    return options;
  });

  // 待分组人员列表（未分组时显示所有人员）
  // const waitingForGroup = computed(() => {

  // });

  // 初始化人员名称
  const initializeNames = () => {
    peopleNames.value = Array.from({ length: totalPeople.value }, (_, i) => `人员${i + 1}`);
  };

  // 保存应用数据
  const saveAppData = async () => {
    try {
      await invoke('save_app_data', {
        cacheGroups: CacheGroups.value,
        waitingGroups: waitingForGroup.value
      });
    } catch (error) {
      console.error('保存数据失败:', error);
    }
  };

  // 加载应用数据
  const loadAppData = async () => {
    try {
      const appData = await invoke('load_app_data') as any;
      console.log(appData)
      if (appData.cache_groups) {
        CacheGroups.value = appData.cache_groups;
        waitingForGroup.value = appData.waiting_groups;
      }

      // 根据总人数自动设置合适的组数
      const total = totalPeople.value;
      if (total >= 8) {
        groupCount.value = 4; // 8人分4组，每组2人
      } else if (total >= 6) {
        groupCount.value = 3; // 6人分3组，每组2人
      } else if (total >= 4) {
        groupCount.value = 2; // 4人分2组，每组2人
      } else {
        groupCount.value = 2; // 默认2组
      }

      console.log('数据加载成功');
    } catch (error) {
      console.error('加载数据失败:', error);
      initializeNames();
    }
  };

  // 随机分组函数
  const randomizeGroups = async () => {
    // 复制人员列表并随机排序
    const shuffledNames = [...waitingForGroup.value].sort(() => Math.random() - 0.5);

    const newGroups: Group[] = [];
    let nameIndex = 0;

    // 创建分组
    for (let i = 0; i < groupCount.value; i++) {
      const groupSize = peoplePerGroup.value + (i < remainingPeople.value ? 1 : 0);
      const groupMembers = shuffledNames.slice(nameIndex, nameIndex + groupSize);
      nameIndex += groupSize;

      newGroups.push({
        id: i + 1,
        members: groupMembers,
        points: 0  // 新增：确保每个组都有初始分值为0
      });
    }

    groups.value = newGroups;
    isRandomized.value = true;
    await showResultInPopup();

    // 分组后保存数据
    // await saveAppData();

    // 自动显示分组结果窗口
    // setTimeout(() => {
    //   showResultInPopup();
    // }, 500);
  };

  // 重新分组
  const regroup = async () => {
    await randomizeGroups();
  };

  // 重置分组
  // const resetGroups = async () => {
  //   groups.value = [];
  //   isRandomized.value = false;

  //   // 先尝试加载本地缓存的数据
  //   try {
  //     const appData = await invoke('load_app_data') as any;
  //     if (appData && appData.people_names && appData.people_names.length === totalPeople.value) {
  //       // 如果缓存数据存在且人数匹配，使用缓存数据
  //       peopleNames.value = appData.people_names;
  //       console.log('重置时加载缓存数据成功');
  //     } else {
  //       // 否则使用默认初始化
  //       initializeNames();
  //       console.log('重置时使用默认数据');
  //     }
  //   } catch (error) {
  //     console.error('重置时加载缓存数据失败:', error);
  //     // 如果加载失败，使用默认初始化
  //     initializeNames();
  //   }
  // };





  // 切换编辑模式
  const toggleEditMode = async () => {
    // if (!editMode.value) {
    //   console.log('从编辑模式切换到查看模式')
    //   const appData = await invoke('load_app_data') as any;
    //   if (appData && appData.CacheGroups && appData.CacheGroups.length) {
    //     CacheGroups.value = appData.CacheGroups;
    //   } else {
    //     CacheGroups.value = [];
    //   }
    // }
    editMode.value = !editMode.value;
  };
  //textarea 添加事件
  const handleAddRolesInput = async () => {
    const input = textareaInput.value.trim();
    if (input) {
      const names = input.split(/[,\s]+/).map(name => name.trim());
      CacheGroups.value = [...new Set([...CacheGroups.value, ...names])];
      textareaInput.value = '';
      await saveAppData();
      await toggleEditMode();
    }
  };
  // 组件挂载时加载数据
  onMounted(async () => {
    await loadAppData();
  });

  // 事件函数
  const addToWaitingGroup = (index: number, name: string) => {
    waitingForGroup.value.push(name);
    peopleNames.value.splice(index, 1);
  }
  const removeFromWaitingGroup = (index: number, name: string) => {
    waitingForGroup.value.splice(index, 1);
    peopleNames.value.push(name);
  }
  // 从缓存组中删除
  const removeFromCacheGroups = async (index: number) => {
    CacheGroups.value.splice(index, 1);
    await saveAppData();
  }

  // 优化后的展示分组结果弹窗函数 - 通过URL参数直接渲染数据
  const showResultInPopup = async () => {
    try {
      // 检查是否有分组数据
      if (groups.value.length === 0) {
        console.warn('没有分组数据可显示');
        return;
      }

      // 将分组数据编码为URL参数
      const groupData = {
        groups: groups.value,
        peoplePerGroup: peoplePerGroup.value,
        remainingPeople: remainingPeople.value
      };

      // 使用 encodeURIComponent 替代 btoa 来处理Unicode字符
      const encodedData = encodeURIComponent(JSON.stringify(groupData));

      // 构建包含数据的URL
      const resultUrl = `#/result?data=${encodedData}`;

      // 先检查是否已经存在结果窗口
      const existingWindows = await getAllWebviewWindows();
      const resultWindow = existingWindows.find(win => win.label === 'result');

      if (resultWindow) {
        try {
          // 如果窗口已存在，显示并发送数据
          await resultWindow.show();
          await resultWindow.setFocus();

          // 对于已存在的窗口，使用事件发送数据（而不是修改URL）
          await emit('group-data', {
            groups: groups.value,
            peoplePerGroup: peoplePerGroup.value,
            remainingPeople: remainingPeople.value
          });
          console.log('已存在窗口，数据发送成功');
          return;
        } catch (windowError) {
          console.warn('操作现有窗口失败，尝试创建新窗口:', windowError);
          // 如果操作现有窗口失败，继续创建新窗口
        }
      }

      // 创建新窗口，直接传递数据
      const webview = new WebviewWindow('result', {
        url: resultUrl,
        title: `分组结果`,
        width: 260,
        height: 375,
        decorations: true,
        center: false,
        resizable: true,
        maximizable: false,
        minimizable: false,
        focus: true
      });

      webview.once('tauri://created', () => {
        console.log('分组结果弹窗创建成功，数据已通过URL参数传递');
      });

      webview.once('tauri://error', (e) => {
        console.error('创建分组结果弹窗失败:', e);
      });

    } catch (error) {
      console.error('创建分组结果弹窗时发生错误:', error);
    }
  };
</script>



<style scoped>
  /* 在原有样式基础上添加新样式 */

  .mode-btn {
    background: #FF9800;
    color: white;
    padding: 6px 12px;
    font-size: 0.85em;
    min-width: auto;
  }

  .mode-btn:hover {
    background: #F57C00;
  }

  /* 人员名单列表样式 */
  .names-list-section {
    width: 100%;
    max-width: 500px;
    margin-bottom: 20px;
  }

  .names-list-section h3 {
    text-align: center;
    margin-bottom: 12px;
    font-size: 1.1em;
  }

  .names-list {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 0 15px 10px 15px;
    height: 320px;
    overflow-y: auto;
    backdrop-filter: blur(10px);
  }

  .name-list-item {
    cursor: pointer;
    /* display: flex; */
    /* align-items: center; */
    padding: 8px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  }

  .name-text {
    color: white;
    text-align: center;
    font-size: 0.95em;
  }

  .arrow-icon {
    font-weight: bold;
    min-width: 25px;
  }

  .changeArrow {
    color: #40ff00;
    margin: auto 10px;
  }

  .remove-icon {
    color: #FF4D4F;
  }

  .add-icon {
    color: #4CAF50;
    margin-right: 10px;
  }


  /* 待分组列表样式 */
  .waiting-list-section {
    width: 100%;
    max-width: 500px;
    margin-bottom: 20px;
  }

  .waiting-list-section h3 {
    text-align: center;
    margin-bottom: 12px;
    font-size: 1.1em;
    color: #4CAF50;
  }

  .waiting-list {
    background: rgba(76, 175, 80, 0.1);
    border-radius: 8px;
    padding: 15px;
    border: 1px solid rgba(76, 175, 80, 0.3);
  }

  .waiting-item {
    display: flex;
    align-items: center;
    padding: 6px 0;
    color: #4CAF50;
  }

  .waiting-number {
    margin-right: 10px;
    font-weight: bold;
  }

  .waiting-text {
    font-size: 0.95em;
  }

  /* 配置区域增强 */
  .config-section {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    background: rgba(255, 255, 255, 0.1);
    padding: 15px;
    border-radius: 8px;
    backdrop-filter: blur(10px);
    flex-wrap: wrap;
    justify-content: center;
  }

  /* 原有样式保持不变，只添加新样式 */
</style>

<style>

  /* 全局样式重置 - 消除浏览器默认边距 */
  html,
  body {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  #app {
    height: 100%;
    width: 100%;
  }

  input,
  textarea {
    outline: none !important;
    border: none !important;
    box-shadow: 0 0 0 0 transparent;
    /* 初始无阴影 */
    transition: box-shadow 0.3s ease-in-out;
    /* 平滑过渡 */
  }

  input:focus-visible,
  textarea:focus-visible {
    outline: none !important;
    border: none !important;
    box-shadow: 0 0 0 2px #dee39d;
    /* 聚焦时蓝色阴影 */
  }

  /* 针对特定容器应用滚动条样式 */
  custom-scroll {
    /* Firefox & 新版Chrome/Safari/Edge */
    /* scrollbar-width: thin; */
    /* 滚动条细模式 */
    /* scrollbar-color: #888 #f1f1f1; */
    /* 滑块颜色 轨道颜色 */
    /* overflow-y: auto; */
    /* max-height: 300px; */
    /* 示例高度 */
  }

  /* Webkit浏览器（Chrome、Safari、旧版Edge） */
  .custom-scroll::-webkit-scrollbar {
    width: 10px;
    /* 垂直滚动条宽度 */
    height: 10px;
    /* 水平滚动条高度 */
  }

  .custom-scroll::-webkit-scrollbar-track {
    background: #f1f1f11a;
    /* 轨道背景色 */
    border-radius: 5px;
  }

  .custom-scroll::-webkit-scrollbar-thumb {
    background: #dcdbdb;
    /* 滑块颜色 */
    border-radius: 5px;
  }

  .custom-scroll::-webkit-scrollbar-thumb:hover {
    background: #929292;
    /* 悬停时颜色 */
  }
</style>

<style scoped>
  .container {
    margin: 0;
    padding: 15px;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100vh;
    width: 100%;
    overflow: hidden;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    box-sizing: border-box;
  }

  h1 {
    text-align: center;
    margin-bottom: 8px;
    font-size: 2em;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  }

  .config-section {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    background: rgba(255, 255, 255, 0.1);
    padding: 15px;
    border-radius: 8px;
    backdrop-filter: blur(10px);
  }

  .config-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .config-item label {
    font-weight: 600;
    font-size: 0.9em;
  }

  .config-item select {
    padding: 6px 10px;
    border: none;
    border-radius: 4px;
    background: white;
    color: #333;
    font-size: 0.9em;
    min-width: 80px;
  }

  .names-section {
    width: 100%;
    max-width: 500px;
    margin-bottom: 20px;
  }

  .names-section h3 {
    text-align: center;
    margin-bottom: 12px;
    font-size: 1.1em;
  }

  .names-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 8px;
    max-height: 120px;
    overflow-y: auto;
  }

  .name-item input {
    width: 100%;
    padding: 6px 10px;
    border: none;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.9);
    color: #333;
    font-size: 0.9em;
    box-sizing: border-box;
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-bottom: 20px;
    flex-wrap: wrap;
    justify-content: center;
  }

  button {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 0.9em;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    min-width: 80px;
  }

  .primary-btn {
    background: #4CAF50;
    color: white;
  }

  .primary-btn:hover:not(:disabled) {
    background: #45a049;
    transform: translateY(-1px);
  }

  .primary-btn:disabled {
    background: #cccccc;
    cursor: not-allowed;
  }

  .secondary-btn {
    background: #2196F3;
    color: white;
  }

  .secondary-btn:hover {
    background: #1976D2;
    transform: translateY(-1px);
  }

  .reset-btn {
    background: #f44336;
    color: white;
  }

  .reset-btn:hover {
    background: #da190b;
    transform: translateY(-1px);
  }

  .groups-section {
    width: 100%;
    max-width: 700px;
    max-height: 400px;
    overflow-y: auto;
  }

  .groups-section h3 {
    text-align: center;
    margin-bottom: 15px;
    font-size: 1.1em;
  }

  .names-list-title {
    text-align: center;
    margin-bottom: 10px;
    font-size: 0.5em;
  }

  .groups-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 15px;
    padding: 5px;
  }

  .group-card {
    background: rgba(255, 255, 255, 0.95);
    color: #333;
    padding: 15px;
    border-radius: 8px;
    box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
    transition: transform 0.3s ease;
    min-height: 120px;
  }

  .group-card:hover {
    transform: translateY(-3px);
  }

  .group-card h4 {
    margin: 0 0 12px 0;
    color: #667eea;
    border-bottom: 1px solid #667eea;
    padding-bottom: 4px;
    font-size: 0.95em;
  }

  .group-card ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .group-card li {
    padding: 3px 0;
    border-bottom: 1px solid #eee;
    font-size: 0.9em;
  }

  .group-card li:last-child {
    border-bottom: none;
  }

  .group-info {
    margin-top: 15px;
    text-align: center;
    font-style: italic;
    opacity: 0.8;
    font-size: 0.9em;
  }

  /* 自定义滚动条 */
  .names-grid::-webkit-scrollbar,
  .groups-section::-webkit-scrollbar {
    width: 6px;
  }

  .names-grid::-webkit-scrollbar-track,
  .groups-section::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .names-grid::-webkit-scrollbar-thumb,
  .groups-section::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
  }

  .names-grid::-webkit-scrollbar-thumb:hover,
  .groups-section::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.5);
  }

  @media (max-width: 768px) {
    .container {
      padding: 10px;
    }

    h1 {
      font-size: 1.6em;
      margin-bottom: 5px;
    }

    .config-section {
      flex-direction: column;
      gap: 12px;
      padding: 12px;
    }

    .names-grid {
      grid-template-columns: repeat(2, 1fr);
      max-height: 100px;
    }

    .actions {
      gap: 8px;
    }

    button {
      padding: 8px 16px;
      font-size: 0.85em;
      min-width: 70px;
    }

    .groups-grid {
      grid-template-columns: 1fr;
      gap: 10px;
    }

    .group-card {
      padding: 12px;
      min-height: 100px;
    }
  }

  @media (max-width: 480px) {
    .names-grid {
      grid-template-columns: 1fr;
      max-height: 150px;
    }

    .groups-section {
      max-height: 300px;
    }
  }

  /* 新增样式：水平布局 */
  .horizontal-layout {
    display: flex;
    min-width: 300px;
    max-width: 1000px;
    gap: 20px;
    margin-bottom: 20px;
    /* background-color: rgb(226, 144, 36); */
  }

  .left-panel,
  .right-panel {
    flex: 1;
  }

  .left-panel .names-section,
  .left-panel .names-list-section,
  .right-panel .waiting-list-section {
    margin-bottom: 0;
  }

  @media (max-width: 768px) {
    .horizontal-layout {
      flex-direction: column;
    }
  }


  /* 待分组文字动画 */
  .impatient {
    font-weight: bold;
    color: #ff4d4d;
    /* 红色系，表达焦急 */
    animation: impatientEffect 0.3s infinite alternate;
    display: inline-block;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  }

  @keyframes impatientEffect {
    0% {
      transform: translate(0, 0) rotate(0deg);
      color: #ff4d4d;
      letter-spacing: 0;
    }

    25% {
      transform: translate(-2px, -2px) rotate(-1deg);
      color: #ff6666;
      letter-spacing: 1px;
    }

    50% {
      transform: translate(2px, 2px) rotate(1deg);
      color: #ff3333;
      letter-spacing: -1px;
    }

    75% {
      transform: translate(-1px, 2px) rotate(0.5deg);
      color: #ff5555;
      letter-spacing: 0.5px;
    }

    100% {
      transform: translate(1px, -2px) rotate(-0.5deg);
      color: #ff4d4d;
      letter-spacing: 0;
    }
  }

  .edit-panel {
    width: 180px;

    .edit-list-item {
      cursor: pointer;
      padding: 8px 0;
      border-bottom: 1px solid rgba(255, 255, 255, 0.2);
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
    }

    .edit-list-item .name-text {
      flex: 1;
      text-align: center;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin-right: auto;
    }

    .edit-list-item .del-icon {
      position: absolute;
      right: 0;
      color: rgb(249, 97, 97);
    }

    .del-icon {
      color: rgb(249, 97, 97);
    }
  }
</style>