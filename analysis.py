import pandas as pd
import matplotlib.pyplot as plt
import matplotlib

# Definir o backend como Agg
matplotlib.use('Agg')

# Carregar os resultados do benchmark para bays29
df_bays29 = pd.read_csv("bays29_benchmark_results.csv")
df_brg180 = pd.read_csv("brg180_benchmark_results.csv")

# Gráfico para bays29
plt.figure(figsize=(14, 6))

# Distância
plt.subplot(1, 2, 1)
plt.plot(df_bays29["Run"], df_bays29["2-opt Distance"], label="2-opt Distance")
plt.plot(df_bays29["Run"], df_bays29["Swap Distance"], label="Swap Distance")
plt.xlabel('Run')
plt.ylabel('Distance')
plt.title('Distances for bays29 Instance')
plt.legend()
plt.grid(True)

# Tempo
plt.subplot(1, 2, 2)
plt.plot(df_bays29["Run"], df_bays29["2-opt Time (µs)"], label="2-opt Time (µs)")
plt.plot(df_bays29["Run"], df_bays29["Swap Time (µs)"], label="Swap Time (µs)")
plt.xlabel('Run')
plt.ylabel('Time (µs)')
plt.title('Times for bays29 Instance')
plt.legend()
plt.grid(True)

plt.tight_layout()
plt.savefig("bays29_benchmark.png")  # Salvar o gráfico como arquivo PNG
plt.close()

# Gráfico para brg180
plt.figure(figsize=(14, 6))

# Distância
plt.subplot(1, 2, 1)
plt.plot(df_brg180["Run"], df_brg180["2-opt Distance"], label="2-opt Distance")
plt.plot(df_brg180["Run"], df_brg180["Swap Distance"], label="Swap Distance")
plt.xlabel('Run')
plt.ylabel('Distance')
plt.title('Distances for brg180 Instance')
plt.legend()
plt.grid(True)

# Tempo
plt.subplot(1, 2, 2)
plt.plot(df_brg180["Run"], df_brg180["2-opt Time (µs)"], label="2-opt Time (µs)")
plt.plot(df_brg180["Run"], df_brg180["Swap Time (µs)"], label="Swap Time (µs)")
plt.xlabel('Run')
plt.ylabel('Time (µs)')
plt.title('Times for brg180 Instance')
plt.legend()
plt.grid(True)

plt.tight_layout()
plt.savefig("brg180_benchmark.png")  # Salvar o gráfico como arquivo PNG
plt.close()
