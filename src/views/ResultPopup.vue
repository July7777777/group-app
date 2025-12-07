<template>
  <div class="group-result-page">
    <div class="result-header">
      <h1>分组结果</h1>
      <div class="result-info">
        <p>总: {{ totalPeople }}人</p>
        <p>分: {{ groupCount }}组</p>
        <p>每组: {{ peoplePerGroup }}人</p>
        <p v-if="remainingPeople > 0">有{{ remainingPeople }}组多1人</p>
      </div>
    </div>

    <div class="groups-container">
      <div
        v-for="group in groups"
        :key="group.id"
        class="group-card"
      >
        <div class="group-header">
          <span>{{ group.id }}组</span>
          <span class="member-count">({{ group.members.length }}人)</span>
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
  // import { getCurrentWindow } from '@tauri-apps/api/window';

  // 定义分组类型
  interface Group {
    id: number;
    members: string[];
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

  // 处理分组数据
  const handleGroupData = (data: any) => {
    console.log('接收到分组数据:', data);

    if (data.groups) {
      groups.value = data.groups;
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
        // 解码Base64数据
        const decodedData = atob(dataParam);
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

  // 组件卸载时取消监听
  onUnmounted(() => {
    if (unlisten) {
      unlisten();
      console.log('分组数据监听器已取消');
    }
  });

  // 关闭窗口
  // const closeWindow = async () => {
  //   try {
  //     const win = getCurrentWindow();
  //     await win.close();
  //   } catch (error) {
  //     console.error('关闭窗口失败:', error);
  //   }
  // };
</script>

<style scoped>
  .group-result-page {
    padding: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba200 100%);

    min-height: 100vh;
    color: white;
    font-family: Arial, sans-serif;
  }

  .result-header {
    text-align: center;
    margin-bottom: 30px;
    background: rgba(255, 255, 255, 0.1);
    padding: 20px;
    border-radius: 10px;
    backdrop-filter: blur(10px);
  }

  .result-header h1 {
    margin: 0 0 15px 0;
    font-size: 2em;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  }

  .result-info {
    display: flex;
    justify-content: center;
    gap: 20px;
    flex-wrap: wrap;
    font-size: 1em;
  }

  .result-info p {
    margin: 5px 0;
    font-weight: 600;
  }

  .groups-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 20px;
    margin-bottom: 30px;
  }

  .group-card {
    width: 100px;
    color: #333;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
    transition: transform 0.3s ease, box-shadow 0.3s ease;

    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 10px 15px 10px 15px;
    /* height: 320px; */
    /* overflow-y: auto; */
    backdrop-filter: blur(10px);
  }



  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
    padding-bottom: 10px;
    border-bottom: 2px solid #c1c1c1;
  }

  .group-header h3 {
    margin: 0;
    color: #ffffff;
    font-size: 1.2em;
  }

  .member-count {
    color: #666;
    font-size: 0.9em;
    font-weight: 600;
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

    color: #fff;
    text-align: center;
    font-size: 0.95em;
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