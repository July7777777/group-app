<template>
  <div class="group-result-page">
    <div class="result-header">
      <div class="tip-text">提示：左键点击加分，右键点击减分</div>
    </div>

    <div class="groups-container">
      <div
        v-for="group in groups"
        :key="group.id"
        class="group-card"
        @click="addPoint(group.id)"
        @contextmenu.prevent="subtractPoint(group.id)"
      >
        <div class="group-header">
          <span>{{ group.id }}组</span>
          <span class="member-count">({{ group.points }}分)</span>
        </div>
        <ul class="member-list">
          <li
            v-for="(member, index) in group.members"
            :key="index"
            class="member-item"
          >
            {{ member }}
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from 'vue';
  import { useRoute } from 'vue-router';
  import { listen } from '@tauri-apps/api/event';

  // 定义分组类型
  interface Group {
    id: number;
    members: string[];
    points: number;
  }

  // 获取路由信息
  const route = useRoute();

  // 响应式数据
  const groups = ref<Group[]>([]);
  const totalPeople = ref(0);
  const groupCount = ref(0);
  const peoplePerGroup = ref(0);
  const remainingPeople = ref(0);

  let unlisten: (() => void) | null = null;

  // 给指定组加一分
  const addPoint = (groupId: number) => {
    const groupIndex = groups.value.findIndex(g => g.id === groupId);
    if (groupIndex !== -1) {
      groups.value[groupIndex].points += 1;
      console.log(`第${groupId}组积分+1，当前积分: ${groups.value[groupIndex].points}`);
    }
  };

  // 给指定组减一分（撤销）
  const subtractPoint = (groupId: number) => {
    const groupIndex = groups.value.findIndex(g => g.id === groupId);
    if (groupIndex !== -1 && groups.value[groupIndex].points > 0) {
      groups.value[groupIndex].points -= 1;
      console.log(`第${groupId}组积分-1，当前积分: ${groups.value[groupIndex].points}`);
    } else if (groupIndex !== -1) {
      console.log(`第${groupId}组积分已为0，无法再减分`);
    }
  };

  // 处理分组数据
  const handleGroupData = (data: any) => {
    console.log('接收到分组数据:', data);

    if (data.groups) {
      // 确保每个组都有points字段，如果没有则默认为0
      groups.value = data.groups.map((group: any) => ({
        ...group,
        points: group.points !== undefined ? group.points : 0  // 修复：明确检查points是否存在
      }));
    }
    if (data.peoplePerGroup) {
      peoplePerGroup.value = data.peoplePerGroup;
    }
    if (data.remainingPeople) {
      remainingPeople.value = data.remainingPeople;
    }

    // 计算总人数和组数
    totalPeople.value = groups.value.reduce((total, group) => total + group.members.length, 0);
    groupCount.value = groups.value.length;

    console.log('数据更新完成:', {
      groups: groups.value.length,
      totalPeople: totalPeople.value,
      groupCount: groupCount.value
    });
  };

  // 从URL参数解析数据
  const parseGroupDataFromURL = () => {
    try {
      const dataParam = route.query.data as string;
      if (dataParam) {
        const decodedData = decodeURIComponent(dataParam);
        const groupData = JSON.parse(decodedData);

        console.log('从URL参数解析的分组数据:', groupData);
        handleGroupData(groupData);
      } else {
        console.warn('URL中没有找到分组数据参数');
      }
    } catch (error) {
      console.error('解析URL参数数据失败:', error);
    }
  };

  // 组件挂载时立即解析数据
  onMounted(async () => {
    console.log('ResultPopup组件已挂载，开始解析URL参数数据');
    parseGroupDataFromURL();

    // 同时监听事件，以便已存在的窗口可以接收新数据
    try {
      unlisten = await listen('group-data', (event) => {
        console.log('接收到group-data事件');
        handleGroupData(event.payload);
      });
      console.log('分组数据监听器已建立');
    } catch (error) {
      console.error('监听分组数据失败:', error);
    }
  });

  // 组件卸载时取消监听 - 使用更安全的方式
  onUnmounted(() => {
    if (unlisten) {
      try {
        unlisten();
        console.log('分组数据监听器已取消');
      } catch (error) {
        console.warn('取消监听时出现错误，可能是权限问题:', error);
        // 忽略取消监听时的错误，不影响应用功能
      }
    }
  });
</script>

<style scoped>
  .group-result-page {
    padding: 13px;
    padding-top: 0;
    /* background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%); */
    background: linear-gradient(135deg, #667eea 0%, #764ba200 100%);
    min-height: 100vh;
    color: #ffffff;
    font-family: Arial, sans-serif;
    user-select: none;
    /* 防止文字被选择 */
    -webkit-user-select: none;
    /* Safari 支持 */
    -moz-user-select: none;
    /* Firefox 支持 */
    -ms-user-select: none;
    /* IE/Edge 支持 */
  }

  .result-header {
    /* font-size: 12px; */
    text-align: center;
  }

  .tip-text {
    padding: 10px 0;
    font-size: 0.6em;
    font-weight: 600;
    color: #e0e0e0;
    font-style: italic;
    text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
  }

  .groups-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 18px;
    margin-bottom: 30px;
  }

  .group-card {
    width: 77px;
    color: #ffffff;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
    transition: transform 0.3s ease, box-shadow 0.3s ease;
    background: rgba(0, 0, 0, 0.4);
    border-radius: 10px;
    padding: 15px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
    padding-bottom: 10px;
    border-bottom: 2px solid rgba(255, 255, 255, 0.3);
  }

  .group-header span:first-child {
    color: #ffffff;
    font-size: 0.9em;
    font-weight: 700;
    text-shadow: 1px 1px 3px rgba(0, 0, 0, 0.5);
  }

  .member-count {
    color: #62e367;
    font-size: 0.9em;
    font-weight: 700;
    text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
  }

  .member-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .member-item {
    padding: 8px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    transition: background-color 0.2s ease;
    color: #ffffff;
    text-align: center;
    font-size: 0.95em;
    font-weight: 500;
    text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
  }

  .member-item:hover {
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }

  @media (max-width: 120px) {
    .group-result-page {
      padding: 15px;
    }

    .result-info {
      flex-direction: column;
      gap: 10px;
      align-items: center;
    }

    .groups-container {
      grid-template-columns: 1fr;
      gap: 15px;
    }

    .group-card {
      padding: 15px;
    }
  }
</style>