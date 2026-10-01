#!/usr/bin/env python3
"""
Papéli - Gerador de Micro-Assets Anos 80 (Micro PNGs para inline CID)
Gera:
  - 10 gradientes verticais de 16x848 px (ultra-leves, ~250 bytes cada)
  - 4 cantoneiras (TL, TR, BR, BL) pré-rotacionadas para cada um dos 10 temas (80x80 px RGBA, ~600 bytes cada)
"""

import os
import math
from PIL import Image, ImageDraw

OUTPUT_DIR = "assets"
os.makedirs(OUTPUT_DIR, exist_ok=True)

def save_rotations(base_img, theme_name):
    # TL: 0 deg
    tl = base_img
    # TR: 90 deg clockwise (ROTATE_270 in PIL rotates 270 counter-clockwise = 90 clockwise)
    tr = base_img.transpose(Image.Transpose.ROTATE_270)
    # BR: 180 deg
    br = base_img.transpose(Image.Transpose.ROTATE_180)
    # BL: 270 deg clockwise (ROTATE_90 in PIL)
    bl = base_img.transpose(Image.Transpose.ROTATE_90)

    tl.save(os.path.join(OUTPUT_DIR, f"corner_{theme_name}_tl.png"), "PNG", optimize=True)
    tr.save(os.path.join(OUTPUT_DIR, f"corner_{theme_name}_tr.png"), "PNG", optimize=True)
    br.save(os.path.join(OUTPUT_DIR, f"corner_{theme_name}_br.png"), "PNG", optimize=True)
    bl.save(os.path.join(OUTPUT_DIR, f"corner_{theme_name}_bl.png"), "PNG", optimize=True)

def make_gradient(theme_name, stops, width=16, height=848):
    img = Image.new('RGB', (width, height))
    for y in range(height):
        t = y / (height - 1)
        for i in range(len(stops) - 1):
            t0, c0 = stops[i]
            t1, c1 = stops[i+1]
            if t0 <= t <= t1:
                factor = (t - t0) / (t1 - t0) if t1 > t0 else 0
                r = int(c0[0] + factor * (c1[0] - c0[0]))
                g = int(c0[1] + factor * (c1[1] - c0[1]))
                b = int(c0[2] + factor * (c1[2] - c0[2]))
                break
        for x in range(width):
            img.putpixel((x, y), (r, g, b))
    img.save(os.path.join(OUTPUT_DIR, f"grad_{theme_name}.png"), "PNG", optimize=True)

# -------------------------------------------------------------
# Desenhos das Cantoneiras (80x80 px RGBA)
# -------------------------------------------------------------

def draw_rosinha_frufru():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Delicado laço frufru com pérola e caudas onduladas
    # Laço nó central
    d.ellipse([30, 30, 50, 50], fill=(236, 72, 153), outline=(190, 24, 93), width=2)
    # Alça esquerda/cima
    d.ellipse([8, 20, 36, 48], fill=(254, 205, 211), outline=(236, 72, 153), width=2)
    d.ellipse([20, 8, 48, 36], fill=(254, 205, 211), outline=(236, 72, 153), width=2)
    # Caudas da fita fluindo para o canto interno
    d.line([(35, 48), (20, 74)], fill=(236, 72, 153), width=4)
    d.line([(48, 35), (74, 20)], fill=(236, 72, 153), width=4)
    # Pequenos brilhos e pérola
    d.ellipse([36, 36, 44, 44], fill=(255, 255, 255, 240))
    d.polygon([(14, 14), (16, 8), (18, 14), (24, 16), (18, 18), (16, 24), (14, 18), (8, 16)], fill=(244, 114, 182, 220))
    return img

def draw_moranguinho():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Morango fofo estilo anos 80
    # Corpo do morango
    d.polygon([(26, 22), (54, 22), (50, 48), (40, 58), (30, 48)], fill=(225, 29, 72), outline=(159, 18, 57), width=2)
    d.ellipse([26, 18, 54, 34], fill=(225, 29, 72), outline=(159, 18, 57), width=2)
    # Folhas verdes (cálice)
    d.polygon([(40, 14), (32, 22), (48, 22)], fill=(34, 197, 94))
    d.polygon([(30, 16), (24, 24), (34, 24)], fill=(22, 163, 74))
    d.polygon([(50, 16), (46, 24), (56, 24)], fill=(22, 163, 74))
    # Cabinho
    d.line([(40, 16), (36, 8)], fill=(21, 128, 61), width=3)
    # Pontinhos dourados/amarelos das sementes
    seeds = [(34, 28), (46, 28), (40, 36), (34, 44), (46, 44), (40, 50)]
    for sx, sy in seeds:
        d.ellipse([sx-1, sy-2, sx+1, sy+2], fill=(254, 240, 138))
    # Florzinha de morango ao lado
    d.ellipse([54, 48, 64, 58], fill=(255, 255, 255), outline=(244, 114, 182), width=1)
    d.ellipse([57, 51, 61, 55], fill=(234, 179, 8))
    return img

def draw_lavanda_sonho():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Ramos delicados de lavanda com estrelas violeta
    d.line([(18, 18), (62, 62)], fill=(74, 222, 128), width=3)
    # Flores ao longo do ramo
    buds = [(24, 22), (22, 28), (32, 30), (30, 36), (42, 40), (40, 46), (50, 48), (48, 54)]
    for bx, by in buds:
        d.ellipse([bx-4, by-6, bx+4, by+6], fill=(168, 85, 247), outline=(126, 34, 206), width=1)
        d.ellipse([bx-2, by-4, bx+2, by+4], fill=(233, 213, 255))
    # Lacinho lilás na base
    d.ellipse([14, 14, 26, 26], fill=(192, 132, 252), outline=(147, 51, 234), width=2)
    d.ellipse([18, 18, 22, 22], fill=(255, 255, 255))
    return img

def draw_nuvem_algodao():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Nuvem fofinha com estrelas douradas
    # Puffs da nuvem
    d.ellipse([20, 24, 44, 48], fill=(224, 242, 254), outline=(56, 189, 248), width=2)
    d.ellipse([34, 14, 60, 40], fill=(255, 255, 255), outline=(56, 189, 248), width=2)
    d.ellipse([46, 26, 68, 48], fill=(224, 242, 254), outline=(56, 189, 248), width=2)
    d.rectangle([28, 32, 60, 46], fill=(255, 255, 255))
    # Olhinhos dorminhocos fofos estilo kawaii anos 80
    d.arc([36, 28, 44, 34], 0, 180, fill=(3, 105, 161), width=2)
    d.arc([48, 28, 56, 34], 0, 180, fill=(3, 105, 161), width=2)
    # Bochechas rosinha
    d.ellipse([32, 33, 38, 37], fill=(253, 164, 175, 180))
    d.ellipse([54, 33, 60, 37], fill=(253, 164, 175, 180))
    # Estrelinha dourada brilhando
    d.polygon([(14, 44), (17, 36), (20, 44), (28, 47), (20, 50), (17, 58), (14, 50), (6, 47)], fill=(250, 204, 21), outline=(202, 138, 4), width=1)
    return img

def draw_menta_fresca():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Cantoneira de folhas de menta vintage com fita verde água
    d.ellipse([20, 20, 44, 44], fill=(52, 211, 153), outline=(5, 150, 105), width=2)
    d.ellipse([10, 16, 26, 38], fill=(167, 243, 208), outline=(5, 150, 105), width=2)
    d.ellipse([16, 10, 38, 26], fill=(167, 243, 208), outline=(5, 150, 105), width=2)
    d.line([(28, 42), (18, 68)], fill=(16, 185, 129), width=3)
    d.line([(42, 28), (68, 18)], fill=(16, 185, 129), width=3)
    d.ellipse([28, 28, 36, 36], fill=(255, 255, 255))
    return img

def draw_caderno_pautado():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Fita washi / marcador de canto de caderno com coraçãozinho
    d.polygon([(0, 0), (60, 0), (0, 60)], fill=(254, 205, 211), outline=(244, 63, 94), width=2)
    # Linhas de costura / pesponto
    d.line([(10, 0), (0, 10)], fill=(244, 63, 94), width=2)
    d.line([(25, 0), (0, 25)], fill=(244, 63, 94), width=2)
    d.line([(40, 0), (0, 40)], fill=(244, 63, 94), width=2)
    # Coraçãozinho central
    d.ellipse([18, 18, 28, 28], fill=(225, 29, 72))
    d.ellipse([24, 18, 34, 28], fill=(225, 29, 72))
    d.polygon([(18, 24), (34, 24), (26, 34)], fill=(225, 29, 72))
    return img

def draw_carta_romantica():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Arabesco vitoriano clássico com botão de rosa carmim
    d.arc([10, 10, 50, 50], 0, 360, fill=(190, 18, 60), width=2)
    d.arc([20, 20, 60, 60], 90, 270, fill=(244, 63, 94), width=2)
    # Rosa vermelha no centro
    d.ellipse([26, 26, 44, 44], fill=(225, 29, 72), outline=(159, 18, 57), width=2)
    d.arc([29, 29, 41, 41], 45, 225, fill=(255, 228, 230), width=2)
    d.arc([31, 31, 39, 39], 225, 45, fill=(255, 255, 255), width=2)
    # Folhinha verde
    d.polygon([(42, 22), (54, 18), (48, 30)], fill=(34, 197, 94), outline=(21, 128, 61), width=1)
    return img

def draw_constelacao_pop():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Estrela pop 8 pontas anos 80 e lua crescente
    # Estrela grande dourada/rosa
    cx, cy = 34, 34
    d.polygon([
        (cx, cy-24), (cx+5, cy-7), (cx+24, cy), (cx+7, cy+5),
        (cx, cy+24), (cx-5, cy+7), (cx-24, cy), (cx-7, cy-5)
    ], fill=(250, 204, 21), outline=(236, 72, 153), width=2)
    # Núcleo brilhante
    d.ellipse([cx-4, cy-4, cx+4, cy+4], fill=(255, 255, 255))
    # Estrelinhas menores secundárias
    d.polygon([(60, 18), (62, 12), (64, 18), (70, 20), (64, 22), (62, 28), (60, 22), (54, 20)], fill=(217, 70, 239))
    d.polygon([(18, 60), (20, 54), (22, 60), (28, 62), (22, 64), (20, 70), (18, 64), (12, 62)], fill=(56, 189, 248))
    return img

def draw_jardim_encantado():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Raminhos entrelaçados com flores campestres
    d.arc([10, 10, 70, 70], 180, 270, fill=(34, 197, 94), width=3)
    # Florzinhas
    flowers = [(24, 46, (244, 63, 94)), (46, 24, (244, 114, 182)), (35, 35, (251, 146, 60))]
    for fx, fy, col in flowers:
        d.ellipse([fx-6, fy-6, fx+6, fy+6], fill=col, outline=(159, 18, 57), width=1)
        d.ellipse([fx-2, fy-2, fx+2, fy+2], fill=(254, 240, 138))
    # Folhinhas verdes
    d.ellipse([16, 28, 24, 34], fill=(74, 222, 128))
    d.ellipse([28, 16, 34, 24], fill=(74, 222, 128))
    return img

def draw_arcoiris_pastel():
    img = Image.new("RGBA", (80, 80), (255, 255, 255, 0))
    d = ImageDraw.Draw(img)
    # Arcos do arco-íris pastel no canto
    colors = [
        (244, 114, 182), # Rosa
        (251, 146, 60),  # Pêssego
        (250, 204, 21),  # Amarelo
        (74, 222, 128),  # Verde
        (56, 189, 248),  # Azul
        (192, 132, 252)  # Lilás
    ]
    for idx, col in enumerate(colors):
        r = 68 - idx * 6
        d.arc([-10, -10, r*2, r*2], 0, 90, fill=col, width=4)
    # Nuvenzinha na ponta do arco
    d.ellipse([46, 46, 68, 68], fill=(255, 255, 255), outline=(192, 132, 252), width=2)
    d.ellipse([54, 42, 74, 62], fill=(255, 255, 255), outline=(192, 132, 252), width=2)
    return img

# -------------------------------------------------------------
# Definição dos 10 Temas e Geração Completa
# -------------------------------------------------------------

THEMES = [
    ("anos80_rosinha_frufru", draw_rosinha_frufru, [
        (0.0, (255, 255, 255)),
        (0.4, (255, 240, 245)),
        (1.0, (255, 228, 230))
    ]),
    ("anos80_moranguinho", draw_moranguinho, [
        (0.0, (255, 255, 255)),
        (0.5, (255, 245, 245)),
        (1.0, (255, 232, 236))
    ]),
    ("anos80_lavanda_sonho", draw_lavanda_sonho, [
        (0.0, (255, 255, 255)),
        (0.5, (250, 245, 255)),
        (1.0, (243, 232, 255))
    ]),
    ("anos80_nuvem_algodao", draw_nuvem_algodao, [
        (0.0, (255, 255, 255)),
        (0.5, (240, 249, 255)),
        (1.0, (224, 242, 254))
    ]),
    ("anos80_menta_fresca", draw_menta_fresca, [
        (0.0, (255, 255, 255)),
        (0.5, (240, 253, 244)),
        (1.0, (220, 252, 231))
    ]),
    ("anos80_caderno_pautado", draw_caderno_pautado, [
        (0.0, (255, 253, 250)),
        (0.5, (255, 251, 245)),
        (1.0, (255, 245, 248))
    ]),
    ("anos80_carta_romantica", draw_carta_romantica, [
        (0.0, (255, 254, 249)),
        (0.5, (254, 252, 232)),
        (1.0, (254, 243, 199))
    ]),
    ("anos80_constelacao_pop", draw_constelacao_pop, [
        (0.0, (255, 255, 255)),
        (0.5, (250, 245, 255)),
        (1.0, (253, 242, 248))
    ]),
    ("anos80_jardim_encantado", draw_jardim_encantado, [
        (0.0, (255, 255, 255)),
        (0.5, (253, 242, 248)),
        (1.0, (240, 253, 244))
    ]),
    ("anos80_arcoiris_pastel", draw_arcoiris_pastel, [
        (0.0, (255, 255, 255)),
        (0.25, (255, 241, 242)),
        (0.50, (254, 243, 199)),
        (0.75, (224, 242, 254)),
        (1.00, (243, 232, 255))
    ]),
]

def main():
    print(f"Gerando micro-assets para {len(THEMES)} temas...")
    total_bytes = 0
    count = 0
    for name, draw_fn, stops in THEMES:
        # Gera cantoneiras
        img = draw_fn()
        save_rotations(img, name)
        # Gera gradiente
        make_gradient(name, stops)
        count += 5
        print(f"✔ Tema '{name}': cantoneiras (TL, TR, BR, BL) + gradiente gerados.")

    # Calcula tamanho total dos assets
    for f in os.listdir(OUTPUT_DIR):
        if f.endswith(".png"):
            total_bytes += os.path.getsize(os.path.join(OUTPUT_DIR, f))

    print(f"\nSucesso: {count} micro-assets gerados em '{OUTPUT_DIR}'! Tamanho total: {total_bytes / 1024:.2f} KB")

if __name__ == "__main__":
    main()
