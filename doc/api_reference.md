# 小红书 API 接口文档

> **项目**: XHS RS Tools  
> **版本**: v0.1.0  
> **最后更新**: 2026-01-10  
> **状态**: 逆向工程进行中

---

## 目录

1. [概述](#概述)
2. [通用规范](#通用规范)
3. [API 接口列表](#api-接口列表)
   - [搜索相关](#搜索相关)
   - [登录相关](#登录相关)
   - [用户相关](#用户相关)
   - [笔记相关](#笔记相关)

---

## 概述

本文档记录小红书 (XHS/Xiaohongshu) Web API 的逆向工程结果，用于指导后续 Rust 工具开发。

### 基础 URL

| 环境 | 域名 |
|------|------|
| API 服务 | `https://edith.xiaohongshu.com` |
| Web 站点 | `https://www.xiaohongshu.com` |

---

## 通用规范

> [!NOTE]
> 以下规范基于已验证的接口总结，随着更多接口测试将持续更新。

### 基础请求头

所有接口均需携带的标准 HTTP 请求头：

| Header | 说明 | 示例值 |
|--------|------|--------|
| `accept` | 接受的响应类型 | `application/json, text/plain, */*` |
| `accept-language` | 语言偏好 | `zh-CN,zh;q=0.9` |
| `origin` | 请求来源 | `https://www.xiaohongshu.com` |
| `referer` | 来源页面 | `https://www.xiaohongshu.com/` |
| `user-agent` | 浏览器标识 | Chrome/Edge 等现代浏览器 UA |

### 签名相关 (待验证)

> [!WARNING]
> 不同接口对签名头的要求可能不同，具体以各接口文档为准。
> 以下字段在部分接口中出现，但并非所有接口都必需。

| Header | 说明 | 是否必需 |
|--------|------|----------|
| `x-s` | 请求签名 | ⚠️ 部分接口需要 |
| `x-s-common` | 通用签名 | ⚠️ 部分接口需要 |
| `x-t` | 时间戳 | ⚠️ 待验证 |
| `x-b3-traceid` | 追踪 ID | ⚠️ 待验证 |
| `x-xray-traceid` | X-Ray 追踪 ID | ⚠️ 待验证 |

### Cookie 参数

| Cookie | 说明 | 必需性 |
|--------|------|--------|
| `a1` | 设备指纹 | ✅ 必需 |
| `webId` | Web 客户端 ID | ✅ 必需 |
| `web_session` | 会话令牌 | 仅登录后接口需要 |
| `gid` | 游客 ID | ⚠️ 待验证 |

### 响应格式

所有接口返回 JSON 格式，基础结构：

```json
{
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": { ... }
}
```

| 字段 | 说明 |
|------|------|
| `code` | 状态码，成功时可能是 `0` 或 `1000`，以 `success` 字段为准 |
| `success` | 布尔值，是否成功（最可靠的判断依据） |
| `msg` | 状态消息 |
| `data` | 业务数据 |

---

## API 接口列表

---

### 搜索相关

#### 1. 猜你想搜 (Query Trending)

**接口描述**: 获取搜索框的"猜你想搜"推荐词列表，用于在用户点击搜索框时显示个性化的搜索建议和热门趋势话题

**使用场景**: 用户首次进入搜索页面或点击搜索框时调用

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `/api/sns/web/v1/search/querytrending` |
| **Method** | `GET` |
| **需要登录** | ❌ 否 |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**请求参数 (Query Parameters)**

| 参数名 | 类型 | 必填 | 说明 | 示例值 |
|--------|------|------|------|--------|
| `source` | string | 是 | 来源页面 | `Explore` |
| `search_type` | string | 是 | 搜索类型 | `trend` |
| `last_query` | string | 否 | 上次搜索词 | 空字符串 |
| `last_query_time` | int | 否 | 上次搜索时间戳 | `0` |
| `word_request_situation` | string | 是 | 请求场景 | `FIRST_ENTER` |
| `hint_word` | string | 否 | 提示词 | 空字符串 |
| `hint_word_type` | string | 否 | 提示词类型 | 空字符串 |
| `hint_word_request_id` | string | 否 | 提示词请求ID | 空字符串 |

**完整请求示例**

```bash
curl --location --request GET 'https://edith.xiaohongshu.com/api/sns/web/v1/search/querytrending?source=Explore&search_type=trend&last_query=&last_query_time=0&word_request_situation=FIRST_ENTER&hint_word=&hint_word_type=&hint_word_request_id=' \
--header 'accept: application/json, text/plain, */*' \
--header 'accept-language: zh-CN,zh;q=0.9' \
--header 'origin: https://www.xiaohongshu.com' \
--header 'priority: u=1, i' \
--header 'referer: https://www.xiaohongshu.com/' \
--header 'sec-ch-ua: "Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24"' \
--header 'sec-ch-ua-mobile: ?0' \
--header 'sec-ch-ua-platform: "Windows"' \
--header 'sec-fetch-dest: empty' \
--header 'sec-fetch-mode: cors' \
--header 'sec-fetch-site: same-site' \
--header 'user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36' \
--header 'x-b3-traceid: 1c027a9a279fd2e9' \
--header 'x-s: XYS_2UQhPsHCH0c1PUhIHjIj2erjwjQhyoPTqBPT49pjHjIj2eHjwjQgynEDJ74AHjIj2ePjwjQTJdPIP/ZlgMrF+BS1ynp7y741GdGh47kr+9lMtUVInoGU8BpN/LTBy9kT49SGN9r9zrTF8FGEzDTxL0H6cFuU/BQNzbkL8r++89SHtURNzfb3a/z0G9zxtAbynfRenemy2sRfL080pbZI4erhnrEQ8SSizgGIygG6/DcEG7cMzn+9tApaLoWh+epb+eb9zMkOydL7+nYjaB8fqeY/nDle+fi9/n83Pn8Oqb+aJAzoJDlw/AYB478x/nM3yfkdPaTtHjIj2ecjwjHjKc==' \
--header 'x-s-common: 2UQAPsHC+aIjqArjwjHjNsQhPsHCH0rjNsQhPaHCH0c1PUhIHjIj2eHjwjQgynEDJ74AHjIj2ePjwjQhyoPTqBPT49pjHjIj2ecjwjHMN0G1+aHVHdWMH0ijP/SjG/80GAz0weSd8nQTqniE+dp6yg8CPgm1JBQTJA+fyBYMPgSEG0qMPeZIPeG9+0PAwaHVHdW9H0ijHjIj2eqjwjHjNsQhwsHCHDDAwoQH8B4AyfRI8FS98g+Dpd4daLP3JFSb/BMsn0pSPM87nrldzSzQ2bPAGdb7zgQB8nph8emSy9E0cgk+zSS1qgzianYt8Lzs/LzN4gzaa/+NqMS6qS4HLozoqfQnPbZEp98QyaRSp9P98pSl4oSzcgmca/P78nTTL08z/sVManD9q9z18np/8db8aob7JeQl4epsPrzsagW3Lr4ryaRApdz3agYDq7YM47HFqgzkanYMGLSbP9LA/bGIa/+nprSe+9LI4gzVPDbrJg+P4fprLFTALMm7+LSb4d+kpdzt/7b7wrQM498cqBzSpr8g/FSh+bzQygL9nSm7qSmM4epQ4flY/BQdqA+l4oYQ2BpAPp87arS34nMQyFSE8nkdqMD6pMzd8/4SL7bF8aRr+7+rG7mkqBpD8pSUzozQcA8Szb87PDSb/d+/qgzVJfl/4LExpdzQ4fRSy7bFP9+y+7+nJAzdaLp/2LSizgr3wLzpag8C2/zQwrRQynP7nSm7cLS9ygGFJURAzrlDqA8c4M8QcA4SL9c78nkmLAzQzg8APFMU/gzn4MpQynzAP7P6q7Yy/fpkpdzjGMpS8pSn49YQ40z0G9LI8nzl49YPwLTALM87JLShJozQ2bk/J7p7/94n47mAq0zi8pL68/8P4fprLo4Bag8dqAbUJ7PlzbkkanD7q9kjJ7PA87QBaLLI8/+c4o8QynQa/7b7/FRl4BYQ2BpA2bm7+FSicnpnLozianW6qMSl4b+oqg4EJpm7zrS9anTQPFSF+BboLAz8N9pk4gzxanSTqDSb/7+/pdzCcfRB/auE4fpxwpk9anSb/D4M4okU4gzVag898/+c4bpOqrTSpSDMqM8+JnRQyrTA2rH7q9Tc4B+14g4Yag8d8nkl47pQyA8SprbBnDSea9px8sRA8SmFpLSh4d+h4g4r+BRL4FSbaBpQ4fpApdmd8pzfcnpx/n4AL7pF8LS3yb4QP9QanSmFqLSeqBkQye8A+flB4gc6N7+gqgzYanYIG9QM4F8oaLEAngmN8p+M4BD6qgz6a/+QpLSi2fMzpdc3Lbm7GLDAP7+3p78Sygb7NF4M4bmQzgk7GdpF+FS3/B4j8nYcagY9q98D8o+f8sRSpSm78FYl4ezQyFzCa/P7q7Yl4bpQcA4AyDzUaLShzrQ620YragY3NFShPo+kLozaGdbF8nRn4ASQ2bDlanTm8gYsP7+34g4aN9Ey/DSbaBr6qgzlJgbF+LSk4LYQ2BV3anSIqDSb+fpDqDTAyFF3pMkc49bQ4S8pJdp7yfQM4MpQzLTSp0Sr+LDAzdYY8rkSzopFwnRl4rlSpnYit9QBJDShGdbEpd4Va/PIq9kAqb4Q2BTr4b87yFSh/rRw4g4fNMmFJLSby/pQyobrag8MJDQl4e+QyLYTaL+dq7YM4ApAzBQ8HjIj2eDjw0qI+eWE+eDMwaIj2erIH0iINsQhP/rjwjQ1J7QTGnIjKc==' \
--header 'x-t: 1768031245784' \
--header 'x-xray-traceid: cdd36fea7bf484b5363c73355ab3c9d4' \
--header 'Cookie: abRequestId=2665260f-2243-569e-88a0-9a56ca0813d0; webBuild=5.6.5; a1=19ba6cc4c89gebmqj96uoivz1pnlbmo3fhhu1yyb750000666339; webId=30e2a560df5731dbf14b7f473fe3fc26; acw_tc=0a4a7a1b17680299690221696e7b6eb737ae15a8da8467cd3a448ba483de91; gid=yjD0KSSKDJijyjD0KSS4SxvvYjkdD6AVjK7lxDW3huyUI7281D6lqy888KKKqqj8q2dW4JWy; web_session=0400698d9e8b450b4e54ee4b573b4bb0f61972; id_token=VjEAAMbGxd7dHnjosNxy2p/Lky8okMNgkWgwreYN7wTygwqLvtAM2sPPf7kB2RghY/NWP3Itpm44W0iXtMscQpTEwTuA3PObUOi7bmxGqvpnBzQRhBRJbGhX6wv2INvVz2KO4yun; unread={%22ub%22:%2269524b7b000000001f00705f%22%2C%22ue%22:%226961fe72000000002103e900%22%2C%22uc%22:29}; websectiga=634d3ad75ffb42a2ade2c5e1705a73c845837578aeb31ba0e442d75c648da36a; sec_poison_id=b1877ef0-464b-463c-894f-d692d3414587; xsecappid=xhs-pc-web; loadts=1768031245125'
```

> [!NOTE]
> 上述示例中的签名 (`x-s`, `x-s-common`) 和 Cookie 值为实际抓包数据，仅供参考格式。
> 实际使用时需要动态生成签名或使用有效的会话 Cookie。

**响应示例**

```json
{
    "data": {
        "word_request_id": "f6628347-6acf-4ed0-9a5c-e6cc3adc5a0e#1768032675671",
        "title": "猜你想搜",
        "queries": [
            {
                "hint_word_request_id": "f6628347-6acf-4ed0-9a5c-e6cc3adc5a0e#1768032675671",
                "title": "咳嗽变异性哮喘诊断",
                "desc": "咳嗽变异性哮喘诊断",
                "type": "firstEnterOther#trendingSavAggRecall#685b658a00000000150211c2#1#0",
                "search_word": "咳嗽变异性哮喘诊断"
            },
            {
                "title": "网盘拉新蜂小推",
                "desc": "网盘拉新蜂小推",
                "type": "firstEnterOther#q2qNextQuery#网盘拉新#2#0",
                "search_word": "网盘拉新蜂小推"
            }
        ],
        "hint_word": {
            "type": "firstEnterOther#trendingSavAggRecall#685b658a00000000150211c2#1#0",
            "search_word": "咳嗽变异性哮喘诊断",
            "hint_word_request_id": "f6628347-6acf-4ed0-9a5c-e6cc3adc5a0e#1768032675671",
            "title": "咳嗽变异性哮喘诊断",
            "desc": "咳嗽变异性哮喘诊断"
        }
    },
    "code": 1000,
    "success": true,
    "msg": "成功"
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | int | 状态码，`1000` 表示成功 |
| `success` | bool | 是否成功 |
| `msg` | string | 状态消息 |
| `data.word_request_id` | string | 请求唯一标识，格式: `UUID#时间戳` |
| `data.title` | string | 模块标题，如 "猜你想搜" |
| `data.queries` | array | 推荐搜索词列表 |
| `data.queries[].title` | string | 搜索词标题 |
| `data.queries[].desc` | string | 搜索词描述 |
| `data.queries[].search_word` | string | 实际搜索关键词 |
| `data.queries[].type` | string | 推荐类型标识，格式: `场景#算法#ID#序号#标志` |
| `data.queries[].hint_word_request_id` | string | 提示词请求ID（可选） |
| `data.hint_word` | object | 当前显示的提示词信息 |

---

### 登录相关

#### 2. 创建登录二维码 (QR Code Create)

**接口描述**: 创建扫码登录的二维码，返回二维码ID和跳转URL

**使用场景**: 用户点击登录按钮时调用，获取二维码供用户使用小红书 App 扫码

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `/api/sns/web/v1/login/qrcode/create` |
| **Method** | `POST` |
| **Content-Type** | `application/json;charset=UTF-8` |
| **需要登录** | ❌ 否 |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**请求参数 (Request Body)**

```json
{
    "qr_type": 1
}
```

| 参数名 | 类型 | 必填 | 说明 | 示例值 |
|--------|------|------|------|--------|
| `qr_type` | int | 是 | 二维码类型 | `1` |

**完整请求示例**

```bash
curl 'https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/create' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: zh-CN,zh;q=0.9' \
  -H 'content-type: application/json;charset=UTF-8' \
  -H 'origin: https://www.xiaohongshu.com' \
  -H 'priority: u=1, i' \
  -H 'referer: https://www.xiaohongshu.com/' \
  -H 'sec-ch-ua: "Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "Windows"' \
  -H 'sec-fetch-dest: empty' \
  -H 'sec-fetch-mode: cors' \
  -H 'sec-fetch-site: same-site' \
  -H 'user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36' \
  -H 'x-b3-traceid: 523a6c2267e872aa' \
  -H 'x-s: XYS_2UQhPsHCH0c1PUhIHjIj2erjwjQhyoPTqBPT49pjHjIj2eHjwjQgynEDJ74AHjIj2ePjwjQTJdPIPAZlg946GLTl4g+GcLW9PbYa2dYr4emk8rQsPFbUPf8awepnJaRx2bSkJLDUyfEG+FDF8sTwy7HM8rkfcDSILgSI+b8sLo8GG9HEL/pjyb4/zez88gL9afRhc0+1Lfpp4FEzLF8gLM+nLoHAzgb/aS8N8bYpG94mPrkHaMY/GfbnzFhl+aT+c9EIqMQCLDkcpnbLP9I7nDT/Jfznnfl0yLLIaSQQyAmOarEaLSzQc0+cpDQ02nQNLjHVHdWFH0ijJ9Qx8n+FHdF=' \
  -H 'x-s-common: 2UQAPsHC+aIjqArjwjHjNsQhPsHCH0rjNsQhPaHCH0c1PUhIHjIj2eHjwjQgynEDJ74AHjIj2ePjwjQhyoPTqBPT49pjHjIj2ecjwjHMN0G1+aHVHdWMH0ijP/SjG/80GAz0weSd8nQTqniE+dp6yg8CPgm1JBQTJA+fyBYMPgSEG0qMPeZIPeG9+0PAwaHVHdW9H0ijHjIj2eqjwjHjNsQhwsHCHDDAwoQH8B4AyfRI8FS98g+Dpd4daLP3JFSb/BMsn0pSPM87nrldzSzQ2bPAGdb7zgQB8nph8emSy9E0cgk+zSS1qgzianYt8Lzs/LzN4gzaa/+NqMS6qS4HLozoqfQnPbZEp98QyaRSp9P98pSl4oSzcgmca/P78nTTL08z/sVManD9q9z18np/8db8aob7JeQl4epsPrzsagW3Lr4ryaRApdz3agYDq7YM47HFqgzkanYMGLSbP9LA/bGIa/+nprSe+9LI4gzVPDbrJg+P4fprLFTALMm7+LSb4d+kpdzt/7b7wrQM498cqBzSpr8g/FSh+bzQygL9nSm7qSmM4epQ4flY/BQdqA+l4oYQ2BpAPp87arS34nMQyFSE8nkdqMD6pMzd8/4SL7bF8aRr+7+rG7mkqBpD8pSUzozQcA8Szb87PDSb/d+/qgzVJfl/4LExpdzQ4fRSy7bFP9+y+7+nJAzdaLp/2LSizLi3wLzpag8C2/zQwrRQynP7nSm7cLS9ygGFJURAzrlDqA8c4M8QcA4SL9c78nkmLAzQzg8APFMU/gzn4MpQynzAP7P6q7Yy/fpkpdzjGMpS8pSn49YQ40z0G9L98p+n4FpIcDkA8db7GDShJozQ2bk/J7p7/94n47mAq0zi8pL68/8P4fprLo4Bag8dqAbUJ7PlzbkkanD7q9kjJ7PA87QBaLLI8/+c4o8QynQa/7b7/FRl4BYQ2BpA2bm7+FSicnpnLozianW6qMSl4b+oqg4EJpm7zrS9anTQPFSF+BboLAz8N9pk4gzxanSTqDSb/7+/pdzCcfR+womAcg+xyfMLanSPn0Qc4BQypd4MagG78p8c4bScqDRSPnl9qM8r/nMQysRAPA4Sq9TM4eW6pd4Oag8O8/mn4MpQyLTSpsRQ4FS9a9pD8DRSzbmFcLSi/d+3qgzh/DbkaDSby/zQPA8APLSd8pzYJ9pDGf4Anpm7/DS3ygpQ2BV7qS8FqLSeGS8Qz/pAPFzryozxJ7+/4gzTaLpCLrQl4r+t2DEAyDGMqAml47+S4g4yaL+8zrSkGMmI4g4UwobFGFSe+9pDqo8AygbF8nMM4URQyLl72dp7pFS9wbbCqAzyaLp98/mY89pDPrRAPM8FLLRn4MkQzpQdagG68/bc4okQyBRS8rFF+FSeLd47apHhaL+aLrSb8npn4g4i/bmFpdzl4BzQ2rYeanWAq98x+np34g4GGnRB4LS9a/QtpdzSJ7pFtFShqLpQP9TsanTbJrS9Po+n//8SyFSUprRl47zQz/zn87pFLokc4e4QyLESpDMz4LS3yA+SGnzAySm7poQl4rRw8pZ9Jp+cyLS3nLYtpdqAanDIqMz0LdpQy9Qrqdb78DS3JBpALocE8p8FtFSindbQ4flDaL+LaFQc4M4QP7L3anW6q9Tn4M4AnDM9aLpMpDSb+g+nze+A8dp7tFSbyMmQcFI6P0mM+rSb2DTonnWFagWMqM+g8o+rpd4waLpDqAbM4b+Qy9+ya/+wq98l4bmQ4dpyqFDROaHVHdWEH0iTP/PU+eZI+AZEwsIj2erIH0iINsQhP/rjwjQ1J7QTGnIjKc==' \
  -H 'x-t: 1768032931736' \
  -H 'x-xray-traceid: cdd37cc7bf4be81c338eef3aa2ad415d' \
  -b 'abRequestId=2665260f-2243-569e-88a0-9a56ca0813d0; webBuild=5.6.5; a1=19ba6cc4c89gebmqj96uoivz1pnlbmo3fhhu1yyb750000666339; webId=30e2a560df5731dbf14b7f473fe3fc26; gid=yjD0KSSKDJijyjD0KSS4SxvvYjkdD6AVjK7lxDW3huyUI7281D6lqy888KKKqqj8q2dW4JWy; xsecappid=xhs-pc-web; acw_tc=0a4a61c717680317925916518e0248728423c3357106a1a9ae58a5a119ac64; websectiga=6169c1e84f393779a5f7de7303038f3b47a78e47be716e7bec57ccce17d45f99; sec_poison_id=58c4a641-1a3d-4456-96ba-df202a908b4c; loadts=1768032929579; web_session=030037ae52ab65c91c1be94c1a2e4adc3e2648' \
  --data-raw '{"qr_type":1}'
```

> [!NOTE]
> 上述示例中的签名 (`x-s`, `x-s-common`) 和 Cookie 值为实际抓包数据，仅供参考格式。

**响应示例**

```json
{
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "qr_id": "388001768033868688",
        "code": "621328",
        "url": "https://www.xiaohongshu.com/mobile/login?qrId=388001768033868688&ruleId=4&xhs_code=621328&timestamp=1768033868720&channel_type=web&component_id=30e2a560df5731dbf14b7f473fe3fc26",
        "multi_flag": 0
    }
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | int | 状态码，`0` 表示成功 |
| `success` | bool | 是否成功 |
| `msg` | string | 状态消息 |
| `data.qr_id` | string | 二维码唯一标识，用于后续轮询登录状态 |
| `data.code` | string | 6位登录验证码 |
| `data.url` | string | 二维码内容URL，需生成二维码图片供用户扫码 |
| `data.multi_flag` | int | 多设备登录标志 |

**URL 参数解析**

`data.url` 中包含的查询参数：

| 参数 | 说明 |
|------|------|
| `qrId` | 二维码ID，与 `data.qr_id` 相同 |
| `ruleId` | 规则ID，固定为 `4` |
| `xhs_code` | 登录验证码，与 `data.code` 相同 |
| `timestamp` | 二维码创建时间戳 |
| `channel_type` | 渠道类型，`web` 表示网页端 |
| `component_id` | 组件ID，与 `webId` Cookie 相同 |

---

#### 3. 轮询二维码状态 (QR Code Status)

**接口描述**: 轮询二维码扫码状态，判断用户是否已扫码/确认登录

**使用场景**: 创建二维码后循环调用此接口，直到用户扫码确认或二维码过期

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `/api/sns/web/v1/login/qrcode/status` |
| **Method** | `GET` |
| **需要登录** | ❌ 否 |
| **轮询间隔** | 约 1-2 秒 |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**请求参数 (Query Parameters)**

| 参数名 | 类型 | 必填 | 说明 | 示例值 |
|--------|------|------|------|--------|
| `qr_id` | string | 是 | 二维码ID，来自create接口 | `74701768034380919` |
| `code` | string | 是 | 验证码，来自create接口 | `288833` |

**特殊请求头**

| Header | 说明 |
|--------|------|
| `x-login-mode;` | 空值请求头，必须携带 |

**完整请求示例**

```bash
curl 'https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/status?qr_id=74701768034380919&code=288833' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: zh-CN,zh;q=0.9' \
  -H 'origin: https://www.xiaohongshu.com' \
  -H 'referer: https://www.xiaohongshu.com/' \
  -H 'user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36' \
  -H 'x-login-mode;' \
  -H 'x-b3-traceid: d80ffca8ecac1fb8' \
  -H 'x-s: [动态签名]' \
  -H 'x-s-common: [动态签名]' \
  -H 'x-t: 1768034398482' \
  -b 'a1=xxx; webId=xxx; ...'
```

**响应示例 (扫码成功，待确认)**

```json
{
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "qr_id": "896281768034534476",
        "code": "874862",
        "url": "https://www.xiaohongshu.com/mobile/login?qrId=896281768034534476&ruleId=4&xhs_code=874862&timestamp=1768034534508&channel_type=web&component_id=30e2a560df5731dbf14b7f473fe3fc26",
        "multi_flag": 0
    }
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | int | 状态码，`0` 表示成功 |
| `success` | bool | 是否成功 |
| `data.qr_id` | string | 二维码ID |
| `data.code` | string | 验证码 |
| `data.url` | string | 二维码URL |
| `data.multi_flag` | int | 多设备标志 |

**登录状态判断**

> [!IMPORTANT]
> 当用户确认登录后，服务端会设置新的 Cookie：
> - `web_session` - 新的会话令牌
> - `id_token` - 新的身份令牌
> 
> 前端通过检测这些 Cookie 的变化来判断登录成功。

---

### 用户相关

#### 4. 获取当前用户信息 (User Me)

**接口描述**: 获取当前登录用户的基本信息

**使用场景**: 登录成功后调用，获取用户头像、昵称等信息用于页面展示

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `/api/sns/web/v2/user/me` |
| **Method** | `GET` |
| **需要登录** | ✅ 是 |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**完整请求示例**

```bash
curl 'https://edith.xiaohongshu.com/api/sns/web/v2/user/me' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: zh-CN,zh;q=0.9' \
  -H 'origin: https://www.xiaohongshu.com' \
  -H 'referer: https://www.xiaohongshu.com/' \
  -H 'user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36' \
  -H 'x-b3-traceid: c229fcfefe758690' \
  -H 'x-s: [动态签名]' \
  -H 'x-s-common: [动态签名]' \
  -H 'x-t: 1768034559591' \
  -b 'web_session=xxx; id_token=xxx; a1=xxx; webId=xxx; ...'
```

**响应示例**

```json
{
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "user_id": "5ceac80d0000000016011e02",
        "red_id": "867387498",
        "nickname": "kayin的自动化生产矩阵",
        "desc": "混沌算法滋养的协议编制员，赛博城墙的泥瓦匠。",
        "gender": 0,
        "guest": false,
        "images": "https://sns-avatar-qc.xhscdn.com/avatar/xxx?imageView2/2/w/360/format/webp",
        "imageb": "https://sns-avatar-qc.xhscdn.com/avatar/xxx?imageView2/2/w/540/format/webp"
    }
}
```

**响应字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.user_id` | string | 用户唯一ID |
| `data.red_id` | string | 小红书号 |
| `data.nickname` | string | 用户昵称 |
| `data.desc` | string | 用户简介 |
| `data.gender` | int | 性别 (0=未知, 1=男, 2=女) |
| `data.guest` | bool | 是否为游客 |
| `data.images` | string | 小尺寸头像URL (360px) |
| `data.imageb` | string | 大尺寸头像URL (540px) |

---

#### 5. 检查用户访问权限 (Lib Access)

**接口描述**: 检查用户对特定功能的访问权限

**使用场景**: 心跳检测或权限验证

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `https://so.xiaohongshu.com/web/lib` |
| **Method** | `GET` |
| **需要登录** | ✅ 是 |
| **重要性** | ⚠️ 非核心，可选 |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**请求参数**

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `user_id` | string | 是 | 用户ID |

**响应示例**

```json
{
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "hasAccess": false
    }
}
```

> [!NOTE]
> 此接口可能用于检查用户是否有特定功能的访问权限，如收藏夹等。
> 非登录核心流程，可作为心跳检测备选。

---

### 安全相关

#### 6. 安全配置 & 签名JS获取 (SBT Source) 🛡️

**接口描述**: 获取前端安全组件配置，包括核心签名算法文件 URL

**使用场景**: 页面初始化时调用，用于加载反爬虫和签名验证的 JavaScript 文件

**基本信息**

| 属性 | 值 |
|------|-----|
| **URL** | `https://as.xiaohongshu.com/api/sec/v1/sbtsource` |
| **Method** | `POST` |
| **Content-Type** | `application/json;charset=UTF-8` |
| **Cookie** | 需要 `a1` 和 `webId` |
| **测试状态** | ✅ 已验证 (2026-01-10) |

**请求参数 (Request Body)**

```json
{
    "callFrom": "web",
    "appId": "xhs-pc-web"
}
```

**响应示例**

```json
{
    "data": {
        "extraInfo": "{\"dsUrl\":\"https://fe-static.xhscdn.com/as/v1/f218/f/public/ds.js\"}",
        "commonPatch": [
            "/fe_api/burdock/v2/note/post",
            "/api/sns/web/v1/comment/post",
            "/api/sns/web/v1/note/like",
            "/api/sns/web/v1/note/collect",
            "/api/sns/web/v1/user/follow",
            "/api/sns/web/v1/feed",
            "/api/sns/web/v1/login/activate",
            "/api/sns/web/v1/note/metrics_report"
        ],
        "xhsTokenUrl": "https://fe-static.xhscdn.com/as/v1/3e44/public/bf7d4e32677698655a5cadc581fd09b3.js",
        "desVersion": "2",
        "validate": true,
        "signUrl": "https://fe-static.xhscdn.com/as/v1/f218/a15/public/04b29480233f4def5c875875b6bdc3b1.js",
        "signVersion": "1",
        "url": "https://fe-static.xhscdn.com/as/v2/fp/962356ead351e7f2422eb57edff6982d.js",
        "reportUrl": "/api/sec/v1/shield/webprofile"
    },
    "code": 0,
    "success": true,
    "msg": "成功"
}
```

**响应字段说明 🔥**

| 字段 | 用途 | 重要性 | 备注 |
|------|------|--------|------|
| `signUrl` | **签名算法文件** | ⭐⭐⭐⭐⭐ | 目前版本: `04b294...3b1.js`，包含 `XYS_` 签名逻辑 |
| `commonPatch` | **需签名接口列表** | ⭐⭐⭐⭐ | 列出了所有必须携带 `x-s` 签名的接口 |
| `xhsTokenUrl` | Token生成算法 | ⭐⭐⭐ | 生成 `x-t` 等 Token |
| `url` | 浏览器指纹 | ⭐⭐⭐ | 反爬虫指纹采集 |
| `dsUrl` | 设备签名 | ⭐⭐⭐ | 与设备指纹相关 |

---

### 笔记相关

> 待补充

---

## 签名算法分析

### x-s 签名

> [!WARNING]
> 签名算法是核心难点。已从 `/api/sec/v1/sbtsource` 接口获取到签名文件 URL。
> 
> **当前版本**: `signUrl` -> `https://fe-static.xhscdn.com/as/v1/f218/a15/public/04b29480233f4def5c875875b6bdc3b1.js`
> **本地备份**: `doc/sign_algorithm.js`

**JS 分析进度**:
- 文件已下载，代码经过严重混淆 (Obfuscated)
- 使用了控制流平坦化 (Control Flow Flattening) 技术
- 包含大量位运算和数组操作
- 核心入口疑似 `window._ace_2267` 或类似全局导出

**已知特征**:
- 签名以 `XYS_` 开头
- 必须包含 `x-t` (时间戳) 和 `x-s-common`
- 针对 `commonPatch` 列表中的接口强制开启签名校验

### x-s-common 签名

### x-s-common 签名

**已知信息**:
- 签名以 `2UQA` 开头
- 长度较长，包含更多设备和环境信息
- 可能包含 Canvas 指纹、WebGL 信息等

---

## 错误码参考

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| `1000` | 成功 | - |
| `-1` | 通用错误 | 检查请求参数 |
| `401` | 未授权 | 检查登录状态和 Cookie |
| `403` | 禁止访问 | 签名可能失效，需重新生成 |
| `300012` | 签名验证失败 | 重新生成 x-s 签名 |

---

## 更新日志

| 日期 | 版本 | 更新内容 |
|------|------|----------|
| 2026-01-10 | v0.1.0 | 初始化文档，添加热门搜索词接口 |

---

## 待办事项

- [ ] 逆向登录二维码获取接口
- [ ] 逆向二维码状态轮询接口
- [ ] 分析 x-s 签名算法
- [ ] 分析 x-s-common 签名算法
- [ ] 测试笔记发布接口
- [ ] 测试图片上传接口
