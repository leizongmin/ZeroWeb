/**
 * This script implements the $262 host provider API (test262-provider.js) for Test262
 * tests running in WPT. It provides the necessary environment for Test262 tests to
 * execute and communicate with the WPT runner.
 *
 * See Test262 INTERPRETING.md for the specification of this API:
 * https://github.com/tc39/test262/blob/main/INTERPRETING.md
 */

function installAPI(global) {
  global.$262 = {
    /**
     * Creates a new ECMAScript Realm (iframe), defines this API on it,
     * and returns the $262 object of the new realm.
     */
    createRealm: function() {
      const iframe = global.document.createElement('iframe');
      iframe.style.display = 'none';

      const container = global.document.body;
      if (!container) {
        // Should not happen.
          throw new Error('Test262 Host API: createRealm() called before document.body was available.');
      }
      container.appendChild(iframe);

      return installAPI(iframe.contentWindow);
    },

    /**
     * Executes a string as an ECMAScript script.
     */
    evalScript: function(src) {
      const script = global.document.createElement('script');
      script.text = src;
      window.__test262_evalScript_error_ = undefined;
      window.__test262_evalScript_active_ = true;
      try {
          global.document.head.appendChild(script);
      } finally {
          window.__test262_evalScript_active_ = false;
      }

      // Errors in the above appendChild bubble up to the global error handler.
      // test262-reporter.js stashes them in a global var for rethrowing.
      if (window.__test262_evalScript_error_) {
          const err = window.__test262_evalScript_error_;
          window.__test262_evalScript_error_ = undefined;
          throw err;
      }
    },

    /**
     * Detaches an ArrayBuffer.
     */
    detachArrayBuffer: function(buffer) {
      // Transfer and thus detach the buffer.
      postMessage(null, '*', [buffer]);
    },

    /**
     * Triggers garbage collection if supported by the host.
     */
    gc: function() {
      try {
        TestUtils.gc();
      } catch (e) {
        throw new Error('Test262 Host API: gc() failed or not supported: ' + (e.message || e));
      }
    },

    /**
     * Reference to the %AbstractModuleSource% constructor.
     */
    AbstractModuleSource: function() {
      throw new Error('Test262 Host API: AbstractModuleSource not available');
    },

    agent: (function () {
      const workers = [];
      let i32a = null;
      const pendingReports = [];

      // Agents call Atomics.wait on this location to sleep.
      const SLEEP_LOC = 0;
      // 1 if the started worker is ready, 0 otherwise.
      const START_LOC = 1;
      // The number of workers that have received the broadcast.
      const BROADCAST_LOC = 2;
      // Each worker has a count of outstanding reports; worker N uses memory
      // location [WORKER_REPORT_LOC + N].
      const WORKER_REPORT_LOC = 3;
      // SAB 报告环形缓冲（ZeroWeb 运行器实现注记，传输层替换不改 $262 契约——
      // INTERPRETING.md 只约定 report 字符串最终经 getReport 送达，不约定载体）：
      // 上游 postMessage 载体依赖页面主线程消息任务派发，而 getReport 的忙等自旋
      // 阻塞主线程——计数已可见而消息未派发时 shift() 落空、报告永久丢失（Chrome
      // wpt.fyi 同型 harness agent 面 112/112 全失败的根因）。共享内存环形队列绕开
      // 消息派发时序：worker 写完数据后 add 发布（原 WORKER_REPORT_LOC 计数语义
      // 保留），主侧按同一原子序读数据。单 worker 单写单读，无 CAS；每 worker 一条
      // 子环（跨 agent 报告顺序上游无契约，per-agent FIFO 保序）。
      const RING_BASE = 64;
      const RING_STRIDE = 4096;
      const RING_MAX_WORKERS = 8;
      const RING_CAP = RING_STRIDE - 2;

      function workerScript(script) {
        return `
          let index;
          let i32a = null;
          let broadcasts = [];
          let pendingReceiver = null;

          function handleBroadcast() {
            if (pendingReceiver && broadcasts.length > 0) {
              pendingReceiver.apply(null, broadcasts.shift());
              pendingReceiver = null;
            }
          };

          self.onmessage = function({data:msg}) {
            switch (msg.kind) {
              case 'start':
                i32a = new Int32Array(msg.sab);
                index = msg.index;
                (0, eval)(\`${script}\`);
                break;

              case 'broadcast':
                Atomics.add(i32a, ${BROADCAST_LOC}, 1);
                broadcasts.push([msg.sab, msg.id]);
                handleBroadcast();
                break;
            }
          };

          function ringSlot(b, k) {
            return b + 2 + (k % ${RING_CAP});
          }

          self.$262 = {
            agent: {
              receiveBroadcast(receiver) {
                pendingReceiver = receiver;
                handleBroadcast();
              },

              report(msg) {
                // 报告入本 worker 子环（[len][UTF-16 码元...]，头/尾单调计单位、
                // 取模落位；空间不足等主侧消费推进），写完再 add 发布计数。
                const s = String(msg);
                const b = ${RING_BASE} + index * ${RING_STRIDE};
                const n = s.length;
                if (n + 1 > ${RING_CAP}) {
                  throw new Error('agent report exceeds ring capacity');
                }
                while (Atomics.load(i32a, b) - Atomics.load(i32a, b + 1) + n + 1 > ${RING_CAP}) {
                  Atomics.wait(i32a, b + 1, Atomics.load(i32a, b + 1), 1);
                }
                const h = Atomics.load(i32a, b);
                Atomics.store(i32a, ringSlot(b, h), n);
                for (let i = 0; i < n; i++) {
                  Atomics.store(i32a, ringSlot(b, h + 1 + i), s.charCodeAt(i));
                }
                Atomics.add(i32a, b, n + 1);
                Atomics.add(i32a, ${WORKER_REPORT_LOC} + index, 1);
              },

              sleep(s) { Atomics.wait(i32a, ${SLEEP_LOC}, 0, s); },

              leaving() {},

              monotonicNow() {
                return (typeof performance === 'object' && performance && performance.now)
                  ? performance.now() : Date.now();
              }
            }
          };`;
      }

      // 报告子环读取（单消费者主侧）：按发布计数逐条读 [len][码元...]，读毕推进尾。
      // 发布（worker add）与写数据同线程序 + seqcst 原子序，读侧见到计数即数据可见。
      function ringSlot(b, k) {
        return b + 2 + (k % RING_CAP);
      }
      function ringRead(index) {
        const b = RING_BASE + index * RING_STRIDE;
        const h = Atomics.load(i32a, b);
        const t = Atomics.load(i32a, b + 1);
        if (h - t < 1) {
          return null;
        }
        const len = Atomics.load(i32a, ringSlot(b, t));
        const units = new Array(len);
        for (let i = 0; i < len; i++) {
          units[i] = Atomics.load(i32a, ringSlot(b, t + 1 + i));
        }
        Atomics.store(i32a, b + 1, t + len + 1);
        return String.fromCharCode.apply(null, units);
      }

      const agent = {
        start(script) {
          if (i32a === null) {
            i32a = new Int32Array(new SharedArrayBuffer(
              (RING_BASE + RING_MAX_WORKERS * RING_STRIDE) * 4));
          }
          if (workers.length >= RING_MAX_WORKERS) {
            // 上游无界；共享内存布局按 8 子环预算，超限显式失败优于静默越界写。
            throw new Error('Test262 Host API: agent start exceeds ring worker budget');
          }
          const w = new Worker(URL.createObjectURL(new Blob([workerScript(script)], {type: 'text/javascript'})));
          w.index = workers.length;
          // 协调面传**裸 SAB**（token 管线物化对象）：Int32Array 视图会被宿主 clone
          // 的 TypedArray 分支按普通视图拷贝脱管（共享内存语义丢失）。worker 侧
          // `new Int32Array(msg.sab)` 重建视图。
          w.postMessage({kind: 'start', sab: i32a.buffer, index: w.index});
          workers.push(w);
        },

        broadcast(sab, id) {
          if (!(sab instanceof SharedArrayBuffer)) {
            throw new TypeError('sab must be a SharedArrayBuffer.');
          }

          Atomics.store(i32a, BROADCAST_LOC, 0);

          for (const w of workers) {
            w.postMessage({kind: 'broadcast', sab: sab, id: id|0});
          }

          while (Atomics.load(i32a, BROADCAST_LOC) != workers.length) {}
        },

        getReport() {
          for (const w of workers) {
            while (Atomics.load(i32a, WORKER_REPORT_LOC + w.index) > 0) {
              const msg = ringRead(w.index);
              if (msg === null) {
                break;
              }
              pendingReports.push(msg);
              Atomics.sub(i32a, WORKER_REPORT_LOC + w.index, 1);
            }
          }

          return pendingReports.shift() || null;
        },

        sleep(s) { Atomics.wait(i32a, SLEEP_LOC, 0, s); },

        monotonicNow() {
          return (typeof performance === 'object' && performance && performance.now)
            ? performance.now() : Date.now();
        }
      };
      return agent;

    })(),
    global: global
  };

  // Ensure print is available in this realm if it's available in the parent.
  global.print = window.print;

  // Placeholder $DONE for non-async tests.
  // Async tests will have this overwritten by doneprintHandle.js.
  // If called in a non-async test, we throw to fail fast and loudly:
  // - If err is passed, throwing it ensures the test fails with that error.
  // - If no err is passed, it signals that a sync test incorrectly tried to use async signaling.
  global.$DONE = function(err) {
    if (err) {
      throw err;
    }
    throw new Error('Test262 Host API: $DONE called in a non-async test.');
  };

  return global.$262;
}

installAPI(window);
