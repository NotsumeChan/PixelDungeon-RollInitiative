package io.github.notsumechan.pdri;

import android.app.NativeActivity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.InputMethodManager;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

/** NativeActivity con el selector de archivos para importar mods (app/src/mods.rs) y el teclado del sistema. */
public class MainActivity extends NativeActivity {
    private static final int PICK_ZIP = 1;
    private static final long MAX = 64L << 20;

    /** Rust (lo registra android_main): una letra escrita en el teclado del sistema. */
    private static native void nativeChar(int c);

    private static void typed(int c) {
        try {
            nativeChar(c);
        } catch (UnsatisfiedLinkError e) {
            // todavía sin registrar: se pierde la letra
        }
    }

    /** Lo llama Rust (JNI): abre o cierra el teclado del sistema (escribir una dirección, una clave, el chat...). */
    public void setKeyboard(boolean on) {
        runOnUiThread(() -> {
            InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            View v = getWindow().getDecorView();
            if (on) {
                v.setFocusable(true);
                v.setFocusableInTouchMode(true);
                v.requestFocus();
                imm.showSoftInput(v, InputMethodManager.SHOW_FORCED);
            } else {
                imm.hideSoftInputFromWindow(v.getWindowToken(), 0);
            }
        });
    }

    /** NativeActivity no le da a la app el carácter de una tecla: se saca acá y va a Rust. Enter, borrar y Atrás
     *  siguen su camino normal (winit). Un texto entero (ACTION_MULTIPLE) llega de una vez. */
    @Override
    public boolean dispatchKeyEvent(KeyEvent e) {
        if (e.getAction() == KeyEvent.ACTION_MULTIPLE && e.getKeyCode() == KeyEvent.KEYCODE_UNKNOWN) {
            String s = e.getCharacters();
            if (s != null) s.codePoints().forEach(MainActivity::typed);
            return true;
        }
        if (e.getAction() == KeyEvent.ACTION_DOWN) {
            int c = e.getUnicodeChar(e.getMetaState());
            if (c > 0 && !Character.isISOControl(c)) typed(c);
        }
        return super.dispatchKeyEvent(e);
    }

    /** Lo llama Rust (JNI): abre el selector de documentos del sistema para elegir un zip. */
    public void pickZip() {
        runOnUiThread(() -> {
            Intent i = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            i.addCategory(Intent.CATEGORY_OPENABLE);
            i.setType("*/*");
            i.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {"application/zip", "application/x-zip-compressed", "application/octet-stream"});
            startActivityForResult(i, PICK_ZIP);
        });
    }

    /** Copia el zip elegido a files/mod_import.zip (vía .part: Rust lo ve sólo completo) y su nombre a mod_import.name. */
    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != PICK_ZIP || result != RESULT_OK || data == null || data.getData() == null) return;
        Uri uri = data.getData();
        new Thread(() -> {
            File dir = getFilesDir();
            File part = new File(dir, "mod_import.part");
            String name = "mod";
            try (Cursor c = getContentResolver().query(uri, new String[] {OpenableColumns.DISPLAY_NAME}, null, null, null)) {
                if (c != null && c.moveToFirst()) name = c.getString(0);
            } catch (Exception e) {
                // sin nombre: "mod"
            }
            try (InputStream in = getContentResolver().openInputStream(uri); OutputStream out = new FileOutputStream(part)) {
                byte[] b = new byte[65536];
                long total = 0;
                int n;
                while ((n = in.read(b)) > 0) {
                    total += n;
                    if (total > MAX) throw new java.io.IOException("too big");
                    out.write(b, 0, n);
                }
                try (OutputStream o = new FileOutputStream(new File(dir, "mod_import.name"))) {
                    o.write(name.getBytes("UTF-8"));
                }
            } catch (Exception e) {
                part.delete();
                return;
            }
            part.renameTo(new File(dir, "mod_import.zip"));
        }).start();
    }
}
