## CNDJ Launcher
Launcher para la Cyber Noche de Juegos (CNDJ).

## Ejemplo
El launcher lee del archivo `launcher.json` para saber cuál juego puede mostrar como disponible. Se asume que en el directorio de cada juego hay una imagen PNG `launcher_hero.png` y una imagen PNG `launcher_logo.png` para mostrar en el launcher.

Al hacer clic en un juego, se cambia el directorio actual al directorio del juego y se intenta correr un script de Bash `launcher_play.sh`.

Se agrega la variable de entorno `CNDJ_NAME` como nombre de jugador para que el script pueda modificar el archivo de configuración de cada juego para usar el nombre proveído en el launcher y la variable de entorno `CNDJ_ADDRESS` como dirección de IP del servidor principal del evento, en el caso de que se pueda y quiera iniciar el juego autoconectandose al servidor.

```
{
    "list": [
        "example/Counter-Strike",
        "example/Xonotic",
        "example/Minecraft",
        "example/AoEII",
        "example/WCIII"
    ],
    "address": "127.0.0.1"
}
```