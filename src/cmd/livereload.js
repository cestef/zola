(function () {
	// Configuration
	const config = {
		reconnectInterval: 1000,
		maxReconnectAttempts: 50,
		cssReloadTimeout: 200,
		wsProtocol: window.location.protocol === "https:" ? "wss:" : "ws:",
		debugMode: false,
	};

	let socket;
	let reconnectAttempts = 0;
	let reconnectTimer = null;
	let cssRefreshTimer = null;
	let isConnected = false;
	let lastScrollPosition = { x: 0, y: 0 };
	let elementScrollPositions = {};

	const statusIndicator = document.createElement("div");
	statusIndicator.id = "zola-live-reload-status";
	statusIndicator.style.cssText = `
      position: fixed;
      bottom: 10px;
      right: 10px;
      width: 12px;
      height: 12px;
      border-radius: 50%;
      background-color: #f44336;
      opacity: 0.7;
      z-index: 9999;
      transition: background-color 0.3s ease;
    `;

	const log = (message, type = "info") => {
		if (!config.debugMode && type !== "error") return;

		const prefix = "🔄 [zola-live-reload]";
		switch (type) {
			case "error":
				console.error(`${prefix} ${message}`);
				break;
			case "warn":
				console.warn(`${prefix} ${message}`);
				break;
			default:
				console.log(`${prefix} ${message}`);
		}
	};

	const updateStatus = (connected) => {
		statusIndicator.style.backgroundColor = connected ? "#4caf50" : "#f44336";
		isConnected = connected;
	};

	const saveScrollPosition = () => {
		lastScrollPosition = {
			x: window.scrollX || window.pageXOffset,
			y: window.scrollY || window.pageYOffset,
		};

		elementScrollPositions = {};
		document.querySelectorAll("[data-preserve-scroll]").forEach((element) => {
			const id = element.id || `zola-scroll-${Math.random().toString(36).slice(2, 9)}`;
			if (!element.id) {
				element.id = id;
			}

			elementScrollPositions[id] = {
				scrollTop: element.scrollTop,
				scrollLeft: element.scrollLeft,
			};

			log(`Saved scroll position for #${id}: ${element.scrollTop}, ${element.scrollLeft}`);
		});

		try {
			sessionStorage.setItem(
				"zola-scroll-position",
				JSON.stringify({
					window: lastScrollPosition,
					elements: elementScrollPositions,
				})
			);
		} catch (e) {
			log("Failed to save scroll position to sessionStorage", "warn");
		}
	};

	// Restore the saved scroll position
	const restoreScrollPosition = () => {
		let position = lastScrollPosition;
		let elements = elementScrollPositions;

		try {
			const stored = sessionStorage.getItem("zola-scroll-position");
			if (stored) {
				const data = JSON.parse(stored);

				if (data.window && data.elements) {
					position = data.window;
					elements = data.elements;
				} else {
					position = data;
				}
			}
		} catch (e) {
			log("Failed to restore scroll position from sessionStorage", "warn");
		}

		if (position) {
			window.scrollTo(position.x, position.y);
			log(`Restored window scroll position: ${position.x}, ${position.y}`);
		}

		setTimeout(() => {
			if (elements) {
				Object.keys(elements).forEach((id) => {
					const element = document.getElementById(id);
					if (element) {
						const pos = elements[id];
						element.scrollTop = pos.scrollTop;
						element.scrollLeft = pos.scrollLeft;
						log(`Restored scroll for #${id}: ${pos.scrollTop}, ${pos.scrollLeft}`);
					}
				});
			}

			s;
			document.querySelectorAll("[data-preserve-scroll]").forEach((element) => {
				if (!element.id) {
					const id = `zola-scroll-${Math.random().toString(36).substr(2, 9)}`;
					element.id = id;
					log(`Assigned new ID to scrollable element: ${id}`);
				}
			});
		}, 100); // FIXME: This timeout is a hack to ensure DOM is fully loaded
	};

	const refreshCSS = (path) => {
		clearTimeout(cssRefreshTimer);

		cssRefreshTimer = setTimeout(() => {
			log(`Refreshing CSS: ${path}`);

			const links = document.querySelectorAll('link[rel="stylesheet"]');
			let refreshed = false;

			// try for specific match first
			links.forEach((link) => {
				const href = link.getAttribute("href");
				if (href && (href.includes(path) || path.includes(href))) {
					const newHref = href.includes("?")
						? href.replace(/\?v=\d+/, `?v=${Date.now()}`)
						: `${href}?v=${Date.now()}`;

					link.setAttribute("href", newHref);
					refreshed = true;
					log(`Updated specific CSS: ${href} -> ${newHref}`);
				}
			});

			// otherwise refresh all CSS
			if (!refreshed) {
				links.forEach((link) => {
					const href = link.getAttribute("href");
					if (href) {
						const newHref = href.includes("?")
							? href.replace(/\?v=\d+/, `?v=${Date.now()}`)
							: `${href}?v=${Date.now()}`;

						link.setAttribute("href", newHref);
						log(`Updated CSS: ${href} -> ${newHref}`);
					}
				});
			}
		}, config.cssReloadTimeout);
	};

	const refreshImages = (path) => {
		log(`Refreshing images: ${path}`);

		const images = document.querySelectorAll("img");
		let refreshed = false;

		images.forEach((img) => {
			const src = img.getAttribute("src");
			if (src && (src.includes(path) || path.includes(src))) {
				const newSrc = src.includes("?")
					? src.replace(/\?v=\d+/, `?v=${Date.now()}`)
					: `${src}?v=${Date.now()}`;

				img.setAttribute("src", newSrc);
				refreshed = true;
				log(`Updated specific image: ${src} -> ${newSrc}`);
			}
		});

		return refreshed;
	};

	// see if we need a full reload or can do partial refresh
	const handleReload = (path) => {
		log(`Handling reload for: ${path}`);

		if (!path) {
			return reloadPage();
		}

		const fileExtension = path.split(".").pop().toLowerCase();

		switch (fileExtension) {
			case "css":
			case "scss":
			case "sass":
				refreshCSS(path);
				return;

			case "jpg":
			case "jpeg":
			case "png":
			case "gif":
			case "svg":
			case "webp":
				if (refreshImages(path)) {
					return;
				}
				break;

			default:
				break;
		}

		reloadPage();
	};

	const reloadPage = () => {
		log("Performing full page reload");

		saveScrollPosition();

		try {
			sessionStorage.setItem("zola-reload-triggered", "true");
		} catch (e) {
			log("Failed to set reload flag in sessionStorage", "warn");
		}

		window.location.reload();
	};

	const connect = () => {
		let wsPort = null;
		const existingLiveReloadScript = document.querySelector('script[src*="livereload.js"]');

		if (existingLiveReloadScript) {
			const srcUrl = new URL(existingLiveReloadScript.src, window.location.origin);
			const portParam = srcUrl.searchParams.get("port");
			log(`Found existing livereload script with port: ${portParam}`);
			if (portParam) {
				wsPort = portParam;
			}
		}

		if (!wsPort) {
			log(
				`No port found in existing livereload script, using default port: ${window.location.port}`
			);
			wsPort = window.location.port;
		}

		const host = window.location.hostname;
		const wsUrl = `${config.wsProtocol}//${host}:${wsPort}`;

		log(`Connecting to WebSocket server at ${wsUrl}`);

		try {
			socket = new WebSocket(wsUrl);

			socket.onopen = () => {
				log("WebSocket connection established");
				reconnectAttempts = 0;
				updateStatus(true);

				socket.send(
					JSON.stringify({
						command: "hello",
						protocols: ["http://livereload.com/protocols/official-7"],
					})
				);
			};

			socket.onclose = () => {
				log("WebSocket connection closed");
				updateStatus(false);
				scheduleReconnect();
			};

			socket.onerror = (error) => {
				log(`WebSocket error: ${error}`, "error");
				updateStatus(false);
			};

			socket.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					log(`Received message: ${JSON.stringify(data)}`);

					if (data.command === "reload") {
						handleReload(data.path);
					} else if (data.command === "hello") {
						log("Received hello from server");
					}
				} catch (e) {
					log(`Error parsing message: ${e}`, "error");
				}
			};
		} catch (e) {
			log(`Error creating WebSocket: ${e}`, "error");
			scheduleReconnect();
		}
	};

	const scheduleReconnect = () => {
		if (reconnectTimer) {
			clearTimeout(reconnectTimer);
		}

		if (reconnectAttempts < config.maxReconnectAttempts) {
			reconnectAttempts++;
			const delay = Math.min(reconnectAttempts * config.reconnectInterval, 10000);

			log(`Scheduling reconnect attempt ${reconnectAttempts} in ${delay}ms`);
			reconnectTimer = setTimeout(connect, delay);
		} else {
			log("Maximum reconnect attempts reached", "error");
		}
	};

	const cleanup = () => {
		if (socket) {
			socket.close();
		}

		if (reconnectTimer) {
			clearTimeout(reconnectTimer);
		}

		if (cssRefreshTimer) {
			clearTimeout(cssRefreshTimer);
		}

		if (statusIndicator.parentNode) {
			statusIndicator.parentNode.removeChild(statusIndicator);
		}
	};

	// Initialize
	const init = () => {
		log("Initializing Zola Live Reload");

		document.body.appendChild(statusIndicator);

		// check if we're reloading from a previous session
		try {
			if (sessionStorage.getItem("zola-reload-triggered") === "true") {
				sessionStorage.removeItem("zola-reload-triggered");
				restoreScrollPosition();
			}
		} catch (e) {
			log("Error checking session storage", "warn");
		}

		connect();

		window.addEventListener("beforeunload", saveScrollPosition);

		// api for external use
		window.ZolaLiveReload = {
			refresh: () => handleReload(),
			toggleDebug: () => {
				config.debugMode = !config.debugMode;
				log(`Debug mode ${config.debugMode ? "enabled" : "disabled"}`);
				return config.debugMode;
			},
			reconnect: () => {
				if (socket) {
					socket.close();
				}
				reconnectAttempts = 0;
				connect();
			},
			getStatus: () => isConnected,
		};

		log("Initialization complete");
	};

	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", init);
	} else {
		init();
	}

	window.addEventListener("unload", cleanup);
})();
