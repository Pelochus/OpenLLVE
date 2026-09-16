package openllve.android.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

private val LightColors = lightColorScheme(
    primary = OpenBlue,
    secondary = OpenAccent,
    tertiary = OpenWarning,
    background = OpenSurface,
    surface = OpenSurface,
    error = OpenError
)

private val DarkColors = darkColorScheme(
    primary = OpenBlueDark,
    secondary = OpenAccent,
    tertiary = OpenWarning,
    background = OpenSurfaceDark,
    surface = OpenSurfaceDark,
    error = OpenError
)

/**
 * Material 3 / Material You theme. Uses dynamic (wallpaper-derived) colors on
 * Android 12+ and falls back to the base palette otherwise.
 */
@Composable
fun OpenLLVETheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    val context = LocalContext.current
    val colorScheme = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
    } else {
        if (darkTheme) DarkColors else LightColors
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = OpenLLVETypography,
        content = content
    )
}
