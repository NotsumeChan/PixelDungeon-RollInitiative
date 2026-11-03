# core/Button.py

Botón estilo `DashboardItem` del original (`TitleScene.java`): ícono arriba, texto debajo, centrados en `rect`.

```python
Button(py.Rect(x, y, 48, 48), icono, action, Font1x.Render("Play"))
```

- `rect`: en pixeles virtuales.
- `action(Delta)`: se llama al **soltar** el click encima del botón, como `onClick` en Java. Si es `None`, no hace nada.
- Mientras está presionado dibuja una versión iluminada (en Java, `brightness(1.5)`; aquí se suma 80 al RGB).
- No tiene hover: el original es táctil.
- Reemplaza la versión anterior, que cargaba un PNG de `Assets/Buttons/` con escala. `Assets/Buttons/StartButton.png`
  quedó sin usar.
- Sin sonido de click por ahora: el audio queda para el final.
