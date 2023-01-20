#include <iostream>

using u8 = unsigned char;
using u64 = unsigned long long;
struct Bestes
{
    u8 laenge;
    u8 letzter_flip;
};
struct BestesMitEnumeration
{
    u64 enumeration;
    Bestes bestes;
};

inline __device__ u64 enumerate_permutation(u8 *permutation, u8 *indeces, u8 len)
{
    for (u8 i = 0; i < len; i++)
    {
        u8 index = (u8)-1;
        for (u8 j = 0; j < len - i; j++)
        {
            if (index != (u8)-1)
                permutation[j - 1] = permutation[j];
            else if (permutation[j] - 1 == i)
                index = j;
        }
        if (index == (u8)-1)
            return (u64)-1;
        indeces[i] = index;
    }
    u64 result = 0;
    u64 fact = 1;
    for (u8 i = 0; i < len; i++)
    {
        if (i > 1)
            fact *= i;
        result += indeces[i] * fact;
    }
    return result;
}

inline __device__ void permutation_by_enumeration(u64 enumeration, u8 *result, u8 *indeces, u8 len, u64 fact)
{
    for (u8 i = 0; i < len; i++)
    {
        fact /= len - i;
        indeces[i] = enumeration / fact;
        enumeration %= fact;
    }
    for (u8 i = len - 1; i < len; i--)
    {
        u8 index = indeces[i];
        memcpy(result + index + 1, result + index, sizeof(u8));
        result[index] = i + 1;
    }
}

inline __device__ void wenden_und_essen(u8 *stapel, u8 *neuer_stapel, u8 len, u8 index)
{
    u8 gegessen = stapel[index];
    for (u8 i = 0; i < index; i++)
    {
        u8 pfannkuchen = stapel[i];
        if (pfannkuchen > gegessen)
            pfannkuchen--;
        neuer_stapel[i] = pfannkuchen;
    }
    for (u8 i = 0; i < len - index - 1; i++)
    {
        u8 pfannkuchen = stapel[len - 1 - i];
        if (pfannkuchen > gegessen)
            pfannkuchen--;
        neuer_stapel[index + i] = pfannkuchen;
    }
}

extern "C" __global__ void run_permutations(Bestes *prior, Bestes *current, BestesMitEnumeration *bestes_gefundene, u8 size, u64 fact)
{
    printf("Fact: %llu\n", fact);
    u64 index = blockIdx.x * blockDim.x + threadIdx.x;
    u64 max_elements = fact / (blockDim.x * gridDim.x) + 1;
    if (index + max_elements >= fact)
        max_elements = fact - index; // check if > or >=

    u64 enumeration;
    u8 indeces[16];
    u8 result[16];
    u8 neuer_stapel_tmp[16];
    Bestes momentan_bestes;
    bestes_gefundene[index].bestes.laenge = (u8)-1;

    for (u64 i = 0; i < max_elements; i++)
    {
        enumeration = index + i;
        permutation_by_enumeration(enumeration, result, indeces, size, fact);
        momentan_bestes.laenge = (u8)-1;

        for (u8 flip = 0; flip < size; flip++)
        {
            wenden_und_essen(result, neuer_stapel_tmp, size, flip);
            u64 neue_permutation = enumerate_permutation(neuer_stapel_tmp, indeces, size - 1);

            Bestes potenziel_bestes = prior[neue_permutation];
            if (momentan_bestes.laenge > potenziel_bestes.laenge)
                momentan_bestes = potenziel_bestes;
        }
        momentan_bestes.laenge++;
        if (momentan_bestes.laenge != 0)
            current[enumeration] = momentan_bestes;
        if (momentan_bestes.laenge < bestes_gefundene[index].bestes.laenge)
        {
            bestes_gefundene[index].bestes = momentan_bestes;
            bestes_gefundene[index].enumeration = enumeration;
        }
    }
}