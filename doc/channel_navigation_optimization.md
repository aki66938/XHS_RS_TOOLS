# 频道签名采集 URL 导航优化尝试记录

**日期**: 2026-01-16  
**状态**: ❌ 尝试失败，需继续研究

## 背景

当前频道签名采集使用 Playwright **点击模式**，存在偶发性不稳定问题（弹窗、点击偏移等）。

## 优化目标

将点击模式改为 **URL 导航模式**，直接导航到带 `channel_id` 参数的 URL：

```
穿搭: https://www.xiaohongshu.com/explore?channel_id=homefeed.fashion_v3
美食: https://www.xiaohongshu.com/explore?channel_id=homefeed.food_v3
彩妆: https://www.xiaohongshu.com/explore?channel_id=homefeed.cosmetics_v3
影视: https://www.xiaohongshu.com/explore?channel_id=homefeed.movie_and_tv_v3
职场: https://www.xiaohongshu.com/explore?channel_id=homefeed.career_v3
情感: https://www.xiaohongshu.com/explore?channel_id=homefeed.love_v3
家居: https://www.xiaohongshu.com/explore?channel_id=homefeed.household_product_v3
游戏: https://www.xiaohongshu.com/explore?channel_id=homefeed.gaming_v3
旅行: https://www.xiaohongshu.com/explore?channel_id=homefeed.travel_v3
健身: https://www.xiaohongshu.com/explore?channel_id=homefeed.fitness_v3
```

## 实验过程

### 第一次尝试

```python
await page.goto(target_url, wait_until="networkidle", timeout=15000)
```

**结果**: 所有频道在 4ms 内完成，只有最后一个频道（健身）成功捕获签名。

### 第二次尝试

增加等待时间：
```python
await page.goto(target_url, wait_until="domcontentloaded", timeout=15000)
await page.wait_for_timeout(3000)
await page.wait_for_load_state("networkidle", timeout=5000)
```

**结果**: 同样失败，分析 `debug_login.log` 发现只有 `home_feed_fitness` 被捕获。

## 失败原因分析

从 `debug_login.log` 确认：

1. URL 导航模式走的是 **SPA 客户端路由**
2. 页面通过 JavaScript 修改 URL，但 **不会触发独立的 home_feed API 请求**
3. 页面可能使用了缓存或复用了已有数据
4. 只有最后一个频道（fitness）因停留时间最长才成功触发 API

## 当前方案

回退到 **点击模式**，并增加稳定性措施：
- 点击后等待 2 秒
- 滚动页面触发内容加载
- 等待 networkidle
- 人工延迟 1-2 秒

## 后续研究方向

1. 分析小红书前端路由机制，确认如何触发 API 请求
2. 检查是否有特定的 JavaScript 事件需要触发
3. 尝试模拟滚动或其他交互来强制刷新数据
4. 研究是否可以直接调用前端 API 刷新逻辑

---

*待后续逆向调试有新发现时补充*
